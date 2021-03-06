# s3du

`s3du` is a tool which lets you know how much space your [AWS S3] buckets are
using according to either [AWS CloudWatch] or AWS S3.

The CloudWatch mode is the cheapest, but least accurate option for getting the
bucket sizes. It is less accurate because there is up to a 24 hour delay on the
real bucket size vs. the size in CloudWatch as CloudWatch is only updated with
S3 bucket sizes [once per day].

The S3 mode is the most accurate but more expensive mode of operation. In this
mode all of the objects in each discovered bucket are listed and their sizes
summed.

## Installation

`s3du` can be installed by running `cargo install s3du`.

## Minimum Supported Rust Version

The MSRV for this project is currently 1.40.0.

## Usage

`s3du` uses the default [AWS credentials] chain. As long as your AWS
credentials are available in some fashion, and your IAM user/role has the
correct permissions simply running `s3du` should return some results.

For example, if you manage your credentials with [`aws-vault`], you might run
`s3du` as follows:

```shell
aws-vault exec s3du-role -- s3du
```

By default, `s3du` will operate in the `us-east-1` region. This can be
overridden either by the `AWS_REGION` environment variable, or the `--region`
CLI argument.

```shell
# Overriding the default AWS region with an environment variable
env AWS_REGION=eu-west-1 s3du

# Overriding the default AWS region with a CLI arg
s3du --region=eu-central-1

# Listing all buckets in S3 mode
s3du --mode=s3

# Listing a specific bucket's non-current object versions in S3 mode
s3du --mode=s3 --object-versions=non-current my-bucket
```

More information on running `s3du` can be found in the man page or via
`s3du --help`.

## Crate Features

The crate has two features, which are both enabled by default.

| Feature      | Purpose                      |
|--------------|------------------------------|
| `cloudwatch` | Enable use of CloudWatch API |
| `s3`         | Enable use of S3 API         |

`s3du` requires at least one of these features be enabled, attempting to
compile the crate with both features disabled will result in compilation
errors.

## AWS CloudWatch and AWS S3 Bucket Size Discrepancies

The CloudWatch and S3 modes will report sizes slightly differently. The
CloudWatch mode will always show the total bucket size, that is, it will show
the size of all current objects versions + non-current object versions. It is
not possible to change this behaviour.

The S3 mode will, by default, only show the bucket size for current object
versions. Command line flags (or environment variables) can be used to change
how the S3 mode operates. With these you can change the S3 mode to operate in
one of 3 ways:

  - All: Show bucket size as the sum of all modes listed below.
  - Current: Show bucket size as the sum of all current object versions, this
    is the default.
  - Multipart: Show bucket size as the sum of all in-progress multipart
    uploads.
  - NonCurrent: Show bucket size as the sum of all non-current object versions.

These can be selected via the `--object-versions` CLI flag if `s3du` was
compiled with the `s3` feature.

## IAM Policies

In order to enable use of `s3du`, your IAM user or role will need one or both
of the following IAM policies attached, depending on which `s3du` modes you
wish to use.

Full examples of these policies are provided as:

  - A CloudFormation template
  - IAM policy JSON files
  - A Terraform module

Under the `aws` directory of the source repository.

### CloudWatch IAM Policy

This policy will enforce HTTPS use and will allow `s3du` access to the AWS
CloudWatch APIs that it requires.

```json
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Sid": "s3du-cloudwatch",
            "Effect": "Allow",
            "Action": [
                "cloudwatch:GetMetricStatistics",
                "cloudwatch:ListMetrics"
            ],
            "Resource": [
                "*"
            ],
            "Condition": {
                "Bool": {
                    "aws:SecureTransport": true
                }
            }
        }
    ]
}
```

### S3 IAM Policy

This policy will enforce HTTPS use and will allow `s3du` access to the AWS S3
APIs that it requires.

```json
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Sid": "s3du-s3",
            "Effect": "Allow",
            "Action": [
                "s3:GetBucketLocation",
                "s3:ListAllMyBuckets",
                "s3:ListBucket",
                "s3:ListBucketMultipartUploads",
                "s3:ListMultipartUploadParts"
            ],
            "Resource": [
                "*"
            ],
            "Condition": {
                "Bool": {
                    "aws:SecureTransport": true
                }
            }
        }
    ]
}
```

## API Usage

Each of the methods for obtaining bucket sizes have different costs within AWS
as they will make differing numbers of API calls, and each API has its own
pricing model which may also vary by region.

### CloudWatch

AWS CloudWatch is the cheapest method of running `s3du`, at the cost of some
accuracy.

The CloudWatch mode of `s3du` will use at least 1 API call to perform the
`ListMetrics` call and at least 1 API call per S3 bucket for the
`GetMetricStatistics` call.

The reason these are listed as "at least 1" is because the API call results
could be paginated if the results lists are sufficiently long. `ListMetrics`
will paginate after 500 results while `GetMetricStatistics` will paginate after
1,440 statistics.

As a basic example, getting bucket sizes for an AWS account with 4 S3 buckets
in it should use 5 API calls total. 1 `ListMetrics` call to discover the
buckets and 4 `GetMetricStatistics` calls (one for each bucket).

### S3

AWS S3 is a more expensive, but more accurate, method of listing bucket sizes.

The S3 mode of `s3du` will use 1 API call to perform the `ListBuckets` API
call, 1 API call per listed bucket to `GetBucketLocation` to discover its
region, 1 API call per listed bucket to `HeadBucket` to make sure we have
access to list the objects, and:

  - at least 1 call to `ListMultipartUploads`, at least 1 call to
    `ListObjectVersions`, and at least 1 call to `ListParts` if in-progress
    multipart uploads are found in the `All` object mode
  - at least 1 call to `ListObjectsV2` per-bucket in the `Current` object
    (default) mode
  - at least 1 call to `ListObjectVersions` per bucket in the `NonCurrent`
    object mode
  - at least 1 call to `ListMultipartUploads` per-bucket in the `Multipart`
    mode with at least 1 call to `ListParts` if any in-progress multipart
    uploads are found

Each of the API calls listed above will return 1,000 objects maximum, if your
bucket has more objects than this, pagination will be required.

For example, let's say we're running in S3 mode getting the sizes of `current`
object versions and our AWS account has 2 buckets.
`bucket-a` (no versioning enabled) has 10,000 objects and `bucket-b`
(versioning enabled) has 32,768 object versions of which 13,720 are current
versions and 19,048 are non-current versions. There is also an in-progress
multipart upload with 2 parts uploaded in `bucket-a`. This would mean:

  - 1 API call to `ListBuckets` for bucket discovery
  - 2 API calls to `GetBucketLocation` for region discovery, 1 for each bucket
  - 2 API calls to `HeadBucket` to check we have access, 1 for each bucket
  - 10 API calls to `ListObjectsV2` for `bucket-a`
  - 14 API calls to `ListObjectsV2` for `bucket-b`

for a total of 29 API calls.

If we were to run `s3du` against the same account a second time, but ask for
the sum of `all` object versions, we'd get the following:

  - 1 API call to `ListBuckets` for bucket discovery
  - 2 API calls to `GetBucketLocation` for region discovery, 1 for each bucket
  - 2 API calls to `HeadBucket` to check we have access, 1 for each bucket
  - 10 API calls to `ListObjectVersions` for `bucket-a`
  - 33 API calls to `ListObjectVersions` for `bucket-b`
  - 1 API call to `ListMultipartUploads` for `bucket-a`
  - 1 API call to `ListMultipartUploads` for `bucket-b`
  - 1 API call to `ListParts` for `bucket-a`

for a total of 51 API calls.

A third run of `s3du` against the same account but asking for the sum of
`non-current` object versions would result in the following:

  - 1 API call to `ListBuckets` for bucket discovery
  - 2 API calls to `GetBucketLocation` for region discovery, 1 for each bucket
  - 2 API calls to `HeadBucket` to check we have access, 1 for each bucket
  - 1 API calls to `ListObjectVersions` for `bucket-a`
  - 33 API calls to `ListObjectVersions` for `bucket-b`

for a total of 39 API calls.

You will notice that the number of API calls to `ListObjectVersions` for
`bucket-b` are the same across both the `all` and `non-current` object versions
requests, this is because any filtering for current vs. non-current objects in
these scenarios must be done by `s3du`. The `ListObjectVersions` API does not
let us specify which object versions we'd like to retrieve.

## S3 Compatible Storage

`s3du` allows a custom endpoint to be specified while running in S3 mode. This
allows use on S3 compatible storage such as [MinIO].

This use case is currently only tested against MinIO and is very basic.

### Example

```shell
env AWS_ACCESS_KEY_ID=minio \
    AWS_SECRET_ACCESS_KEY=miniosecret \
    s3du --mode=s3 --endpoint=https://minio.example.org/
```

<!-- links -->
[`aws-vault`]: https://github.com/99designs/aws-vault/
[once per day]: https://docs.aws.amazon.com/AmazonS3/latest/dev/cloudwatch-monitoring.html
[AWS credentials]: https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-configure.html
[AWS CloudWatch]: https://aws.amazon.com/cloudwatch/
[AWS S3]: https://aws.amazon.com/s3/
[MinIO]: https://min.io/
