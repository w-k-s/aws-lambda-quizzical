# Quizzical API v4

## 1. Introduction

This repository contains the source code of the Quizzical REST API v3.
The API is used by the [Quizzical Android application](https://github.com/w-k-s/Android-Quizzical).

## 2. History

The Quizzical app and API were originally developed in 2012 as part of a university coursework.
I've continued to work on these projects in order to try out new tools on Android and on the backend.

The advantage of using this project as a testbed is that it only contains two endpoints (`getCategories` and `getQuestions`) so changes can be made easily, usually in one day.

- **Version 1** (Original Coursework): A single file `questions.json` returned categories and questions.
- **[Version 2 (2013)](https://github.com/w-k-s/quizzical-v2)**: API was developed in Go, hosted on Google Cloud and backed by AppEngine DataStore.
- **[Version 3 (2018)](https://github.com/w-k-s/Rust-QuizzicalAPI)**: API was rewritten in Rust, hosted on AWS-EC2 and backed by MongoDB on mLab.
- **Version 4 (2018)** (This Repository): Source code from version 3 was reused to create a serveress API using AWS lambdas. Switched to PostgreSQL over MongoDB.

## 3. Deployment

Amazon has provided a lambda runtime for AWS: [link](https://github.com/awslabs/aws-lambda-rust-runtime).

This project contains two lambdas `categories_lambda.rs` and `questions_lambda.rs`. In order to publish a rust lamda, it must be compiled into a static x86_64 linux binary. The binary must be named `boostrap` and be put in a `zip` file. The `zip` file can then be uploaded as a lambda from the AWS console.

The `make build` command generates two files: `categories.zip` and `questions.zip`, each containing a binary named `bootstrap`. Each zip file is uploaded as a seperate lambda function on AWS.

## 4. Building

To build this project on OS X, you'll need to install `musl-cross`, which is what this project uses to cross-compile from OS-X to x86_64 linux.

```
brew install filosottile/musl-cross/musl-cross
```

For further information on cross compilation, see Useful Resources below.

## 5. Testing

### 5.1 Setting up test database

```
CREATE TABLE categories(
    name VARCHAR(256) PRIMARY KEY,
    active BOOL NOT NULL DEFAULT FALSE
);

CREATE TABLE questions(
    id BIGSERIAL PRIMARY KEY,
    text TEXT NOT NULL,
    category VARCHAR(256) NOT NULL REFERENCES categories(name) ON DELETE cascade
);

CREATE TABLE choices(
    id BIGSERIAL PRIMARY KEY,
    text TEXT NOT NULL,
    correct BOOL NOT NULL DEFAULT false,
    question_id BIGSERIAL NOT NULL REFERENCES questions(id) ON DELETE cascade
);
```

### 5.2 Set

To run a specific test case using the test database, execute the following command from terminal:

```
TEST_CONN_STRING='postgres://<username>:<password>@localhost:5432/quizzicaldb_test' cargo test -- --nocapture test_save_question
```

## 6. Useful Resources

1. [Cross-Compilation 1](https://chr4.org/blog/2017/03/15/cross-compile-and-link-a-static-binary-on-macos-for-linux-with-cargo-and-rust/)

2. [Cross-Compilation 2](https://aws.amazon.com/blogs/opensource/rust-runtime-for-aws-lambda/)
