// s3du: A tool for informing you of the used space in AWS S3.
#![forbid(unsafe_code)]
#![deny(missing_docs)]
use anyhow::{
    Result,
};
use async_trait::async_trait;
use log::debug;
use rusoto_s3::{
    ListBucketsOutput,
    ListObjectsV2Request,
    ListObjectVersionsRequest,
    Object,
    ObjectVersion,
    S3,
    S3Client,
};
use super::common::{
    BucketNames,
    BucketSizer,
    ClientConfig,
    S3ObjectVersions,
};

struct BucketList(Vec<String>);

impl From<ListBucketsOutput> for BucketList {
    fn from(output: ListBucketsOutput) -> Self {
        let mut bucket_names = vec![];

        let buckets = match output.buckets {
            Some(buckets) => buckets,
            None          => vec![],
        };

        for bucket in buckets {
            if let Some(name) = bucket.name {
                bucket_names.push(name);
            }
        }

        BucketList(bucket_names)
    }
}

impl BucketList {
    fn bucket_names(&self) -> &BucketNames {
        &self.0
    }
}

// A RefCell is used to keep the external API immutable while we can change
// metrics internally.
pub struct Client {
    client:          S3Client,
    buckets:         Option<BucketList>,
    object_versions: S3ObjectVersions,
}

#[async_trait]
impl BucketSizer for Client {
    // Return a list of S3 bucket names from CloudWatch.
    async fn list_buckets(&mut self) -> Result<BucketNames> {
        let bucket_list: BucketList = self.client.list_buckets().await?.into();
        let bucket_names            = bucket_list.bucket_names().to_owned();

        self.buckets = Some(bucket_list);

        Ok(bucket_names)
    }

    // Get the size of a given bucket
    async fn bucket_size(&self, bucket: &str) -> Result<usize> {
        debug!("bucket_size: Calculating size for '{}'", bucket);

        let mut size: usize = 0;

        let objects = self.list_objects(bucket).await?;

        for object in objects {
            if let Some(s) = object.size {
                size += s as usize;
            }
        }

        debug!(
            "bucket_size: Calculated bucket size for '{}' is '{}'",
            bucket,
            size,
        );

        Ok(size)
    }
}

impl Client {
    // Return a new CloudWatchClient in the specified region.
    pub fn new(config: ClientConfig) -> Self {
        let region = config.region;

        debug!(
            "new: Creating S3Client in region '{}'",
            region.name(),
        );

        let client = S3Client::new(region);

        Client {
            client:          client,
            buckets:         None,
            object_versions: config.s3_object_versions,
        }
    }

    // List object versions and filter according to S3ObjectVersions
    async fn list_object_versions(&self, bucket: &str) -> Result<Vec<ObjectVersion>> {
        let mut next_key_marker        = None;
        let mut next_version_id_marker = None;
        let mut objects                = vec![];

        loop {
            let input = ListObjectVersionsRequest {
                bucket:            bucket.into(),
                key_marker:        next_key_marker.to_owned(),
                version_id_marker: next_version_id_marker.to_owned(),
                ..Default::default()
            };

            let output = self.client.list_object_versions(input).await?;

            if let Some(versions) = output.versions {
                objects.extend(versions);
            }

            if let Some(truncated) = output.is_truncated {
                if truncated {
                    next_key_marker        = output.next_key_marker;
                    next_version_id_marker = output.next_version_id_marker;
                }
                else {
                    break;
                }
            }
        }

        Ok(objects)
    }

    // This is currently bad, the objects vec could be huge
    async fn list_current_objects(&self, bucket: &str) -> Result<Vec<Object>> {
        let mut continuation_token = None;
        let mut objects            = vec![];

        // Loop until all objects are processed.
        loop {
            let input = ListObjectsV2Request {
                bucket:             bucket.into(),
                continuation_token: continuation_token.to_owned(),
                ..Default::default()
            };

            let output = self.client.list_objects_v2(input).await?;

            if let Some(contents) = output.contents {
                objects.extend(contents);
            }

            // If the output was truncated (Some(true)), we should have a
            // next_continuation_token.
            // If it wasn't, (Some(false) | None) we're done and can break.
            match output.is_truncated {
                Some(true) => {
                    let nct = output.next_continuation_token;
                    continuation_token = nct;
                },
                _ => break,
            }
        }

        Ok(objects)
    }

    // A wrapper to call the appropriate bucket listing functions
    async fn list_objects(&self, bucket: &str) -> Result<Vec<Object>> {
        self.list_current_objects(bucket).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rusoto_mock::{
        MockCredentialsProvider,
        MockRequestDispatcher,
        MockResponseReader,
        ReadMockResponse,
    };
    use rusoto_s3::{
        Bucket,
        Owner,
    };
    use tokio::runtime::Runtime;

    // Possibly helpful while debugging tests.
    fn init() {
        // Try init because we can only init the logger once.
        let _ = pretty_env_logger::try_init();
    }

    // Create a mock S3 client, returning the data from the specified
    // data_file.
    fn mock_client(
        data_file: Option<&str>,
    ) -> Client {
        let data = match data_file {
            None    => "".to_string(),
            Some(d) => MockResponseReader::read_response("test-data", d.into()),
        };

        let client = S3Client::new_with(
            MockRequestDispatcher::default().with_body(&data),
            MockCredentialsProvider,
            Default::default()
        );

        Client {
            client:          client,
            buckets:         None,
            object_versions: S3ObjectVersions::Current,
        }
    }

    #[test]
    fn test_bucketlist_from() {
        let buckets = vec![
            Bucket {
                creation_date: Some("2020-03-12T14:45:00.000Z".into()),
                name:          Some("a-bucket".into()),
            },
            Bucket {
                creation_date: Some("2020-03-11T14:45:00.000Z".into()),
                name:          Some("another-bucket".into()),
            },
        ];

        let owner = Owner {
            display_name: Some("aws".into()),
            id:           Some("1936a5d8a2b189cda450d1d1d514f3861b3adc2df515".into()),
        };

        let output = ListBucketsOutput {
            buckets: Some(buckets),
            owner:   Some(owner),
        };

        let bucket_list: BucketList = output.into();
        let mut bucket_names = bucket_list.bucket_names().to_owned();
        bucket_names.sort();

        let expected = vec![
            "a-bucket",
            "another-bucket",
        ];

        assert_eq!(bucket_names, expected);
    }

    #[test]
    fn test_list_buckets() {
        init();

        let expected = vec![
            "a-bucket-name",
            "another-bucket-name",
        ];

        let mut client = mock_client(
            Some("s3-list-buckets.xml"),
        );
        let mut ret = Runtime::new()
            .unwrap()
            .block_on(Client::list_buckets(&mut client))
            .unwrap();
        ret.sort();

        assert_eq!(ret, expected);
    }

    #[test]
    fn test_list_objects() {
        init();

        let mut client = mock_client(
            Some("s3-list-objects.xml"),
        );

        let ret = Runtime::new()
            .unwrap()
            .block_on(Client::list_objects(&mut client, "test-bucket"))
            .unwrap();

        let owner = Owner {
            display_name: Some("aws".into()),
            id:           Some("1936a5d8a2b189cda450d1d1d514f3861b3adc2df515".into()),
        };

        let expected = vec![
            Object {
                e_tag:         Some("\"1d921b22129502cbbe5cbaf2c8bac682\"".into()),
                key:           Some("file1".into()),
                last_modified: Some("2020-03-12T11:04:09.000Z".into()),
                owner:         Some(owner.to_owned()),
                size:          Some(1024),
                storage_class: Some("STANDARD".into()),
            },
            Object {
                e_tag:         Some("\"1d921b22129502cbbe5cbaf2c8bac682\"".into()),
                key:           Some("file2".into()),
                last_modified: Some("2020-03-10T11:05:09.000Z".into()),
                owner:         Some(owner.to_owned()),
                size:          Some(32768),
                storage_class: Some("STANDARD".into()),
            },
        ];

        assert_eq!(ret, expected);
    }

    #[test]
    fn test_bucket_size() {
        init();

        let client = mock_client(
            Some("s3-list-objects.xml"),
        );

        let bucket = "test-bucket";
        let ret = Runtime::new()
            .unwrap()
            .block_on(Client::bucket_size(&client, bucket))
            .unwrap();

        let expected = 33792;

        assert_eq!(ret, expected);
    }
}
