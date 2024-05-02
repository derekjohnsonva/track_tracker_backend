# Track Tracker Backend

## Running the App

We are running a local dynamoDB using docker.
Make sure you have docker installed then run `docker pull amazon/dynamodb-local` to get the dynamoDB image.
To start the image `docker run -p 8000:8000 amazon/dynamodb-local`.
Install the AWS [cli](https://aws.amazon.com/cli/)
Set your cli credentials by making an IAM user.
Run the following command to make sure that you can access the docker dynamoDB database

```bash
aws dynamodb list-tables --endpoint-url http://localhost:8000
```

Run this command to create the races table

```
aws dynamodb create-table \
    --table-name races \
    --attribute-definitions \
        AttributeName=Id,AttributeType=S \
    --key-schema \
        AttributeName=Id,KeyType=HASH \
    --provisioned-throughput ReadCapacityUnits=1,WriteCapacityUnits=1 \
    --table-class STANDARD \
    --endpoint-url http://localhost:8000
```

You must provide an AWS Region and credentials, but they don't have to be valid. One way to do this is by providing a localstack profile in your config file (~/.aws/config on macOS and Linux; %userprofile%\.aws\config on Windows), as shown.

```
[profile localstack]
region = us-east-1
aws_access_key_id = AKIDLOCALSTACK
aws_secret_access_key = localstacksecret
```

Then you set AWS_PROFILE=localstack when running your application by running `export AWS_PROFILE=localstack`

After this, we can start our app using `cargo run`

## What our backend needs to do

### Competitions

Places where athletes will compete

- [ ] Add/delete/modify a competition
- [ ] Get a batch of competitions using a batch of `competition_id`

### Athlete

A user that has a competition schedule

- [ ] Create an athlete
- [ ] Add/remove/modify an event from their calendar

### Event

An event that an athlete will compete in at a competition

### Users

A generic user type. Should have some sort of credentialed sign in. An Athlete is an extension of the Users Type

- [ ] Add/delete/modify a user
- [ ] add/remove athletes from a following list
- [ ] get all upcoming competitions from all following athletes
