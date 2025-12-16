# UpdateManager
Server and client for distributing software by automatically updating it

## Install (Server)

1. Download the server executable
2. Run the server. It creates all necessary files.
3. Configure your server in the upman.json and put your files you want to distribute in the updates folder.
4. Run the server

## Install (Client)

### Configure it by yourself

1. Download the client executable
2. Run the client. It creates all necessary files.
3. Configure your client in the upman.json
4. Run the client

### Pre-configured
1. Download the upman.json and client executable
2. Run it

### Example Configuration

```json
{
  "port": "4455",
  "address": "127.0.0.1",
  "program": "java",
  "arg0": "-jar",
  "arg1": "./explorer.jar"
}
```
With this configuration, the client would connect to the localhost server. After it received all files, it will run following command

```bash
java -jar ./explorer.jar
```

## Usage (Server)

### Example

## Usage (Client)

### Example
