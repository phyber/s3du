// Implement the BucketSizer trait for the s3::Client
#![forbid(unsafe_code)]
#![deny(missing_docs)]
use anyhow::Result;
use async_trait::async_trait;
use crate::common::{
    Bucket,
    Buckets,
    BucketSizer,
};
use log::debug;
use super::client::Client;

#[async_trait]
impl BucketSizer for Client {
    /// Return `Buckets` discovered in S3.
    ///
    /// This list of buckets will also be filtered by the following:
    ///   - The `bucket` argument provided on the command line
    ///   - The `Region`, ensuring it's in our currently selected `--region`
    async fn buckets(&mut self) -> Result<Buckets> {
        let mut bucket_names = self.list_buckets().await?;

        // If we were provided with a specific bucket name on the CLI, filter
        // out buckets that don't match.
        if let Some(bucket_name) = self.bucket_name.as_ref() {
            bucket_names.retain(|b| b == bucket_name);
        }

        let mut buckets = Buckets::new();

        for bucket in &bucket_names {
            let region = self.get_bucket_location(&bucket).await?;

            // We can only ListBucket for the region our S3 client is in, so
            // we filter for that region here.
            if region == self.region {
                let bucket = Bucket {
                    name:          bucket.into(),
                    region:        Some(region),
                    storage_types: None,
                };

                buckets.push(bucket);
            }
        }

        // Finally, we have a list of buckets that we should be able to get the
        // size for.
        Ok(buckets)
    }

    /// Return the size of `bucket`.
    async fn bucket_size(&self, bucket: &Bucket) -> Result<usize> {
        let name = &bucket.name;
        debug!("bucket_size: Calculating size for '{}'", name);

        let size = self.size_objects(name).await?;

        debug!(
            "bucket_size: Calculated bucket size for '{}' is '{}'",
            name,
            size,
        );

        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::S3ObjectVersions;
    use pretty_assertions::assert_eq;
    use rusoto_core::Region;
    use rusoto_mock::{
        MockCredentialsProvider,
        MockRequestDispatcher,
        MockResponseReader,
        ReadMockResponse,
    };
    use rusoto_s3::S3Client;
    use tokio::runtime::Runtime;

    // Create a mock S3 client, returning the data from the specified
    // data_file.
    fn mock_client(
        data_file: Option<&str>,
        versions:  S3ObjectVersions,
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
            bucket_name:     None,
            object_versions: versions,
            region:          Region::UsEast1,
        }
    }

    #[test]
    #[ignore]
    fn test_buckets() {
        let expected = vec![
            "a-bucket-name",
            "another-bucket-name",
        ];

        let mut client = mock_client(
            Some("s3-list-buckets.xml"),
            S3ObjectVersions::Current,
        );

        let buckets = Runtime::new()
            .unwrap()
            .block_on(Client::buckets(&mut client))
            .unwrap();

        let mut buckets: Vec<String> = buckets.iter()
            .map(|b| b.name.to_owned())
            .collect();

        buckets.sort();

        assert_eq!(buckets, expected);
    }

    #[test]
    fn test_bucket_size() {
        let client = mock_client(
            Some("s3-list-objects.xml"),
            S3ObjectVersions::Current,
        );

        let bucket = Bucket {
            name:          "test-bucket".into(),
            region:        None,
            storage_types: None,
        };

        let ret = Runtime::new()
            .unwrap()
            .block_on(Client::bucket_size(&client, &bucket))
            .unwrap();

        let expected = 33792;

        assert_eq!(ret, expected);
    }
}
