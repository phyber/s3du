# s3du

`s3du` is a tool which lets you know how much space your AWS S3 buckets are
using according to AWS CloudWatch.

Because `s3du` uses CloudWatch to obtain the bucket size, this means that there
could be up to a 24 hour latency on the reported size, vs. the actual size.
This is because CloudWatch is only updated with S3 bucket sizes once per day.

In the future, an alternate mode to iterate over objects in a bucket and sum
the sizes may become available.

## Usage

`s3du` uses the default AWS credentials chain. As long as your AWS credentials
are available in some fashion, and your IAM user/role has the correct
permissions simply running `s3du` should return some results.
