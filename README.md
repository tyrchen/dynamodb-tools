# DynamoDB tester

As AWS provided [DynamoDB local](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/DynamoDBLocal.html), we could leverage it in the tests. However, managing the dynamodb client and tables is tedious, we need to clean that up at the end of every test to not pollute other tests. This crate will help you to create tables with unique names and then tear them down after test ends (by using Drop trait if you ask).

## Usage

First you need to download and run dynamodb local yourself. For example, I unzipped it in ~/bin/dynamodb_local_latest, so I can start it like this:

```bash
$ java -Djava.library.path=~/bin/dynamodb_local_latest/DynamoDBLocal_lib -jar ~/bin/dynamodb_local_latest/DynamoDBLocal.jar -inMemory -sharedDb
```

Feel free to make it a service so that it starts automatically when system starts.

In your test code, you could use it like this:

```rust
// first, create the LocalClient
use dynamodb_tester::LocalClient;
let lc = LocalClient::try_new(8080, "users", None).await?;
let (client, table_name) = lc.inner();
// then you could use the returned client & table_name
// to interact with dynamodb local.
let ret = client
    .put_item()
    .table_name(table_name)
    .set_item(Some(item))
    .send()
    .await?;
```

If you want to integrate it with github action, you could use [this action](https://github.com/rrainn/dynamodb-action):

```yaml
- name: Setup DynamoDB Local
  uses: rrainn/dynamodb-action@v2.0.1
  with:
  port: 8000
  cors: '*'
```

See [build.yml](.github/workflows/build.yml) for more details.
