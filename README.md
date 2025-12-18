# UpdateManager
Server and client for distributing software with automatic updates.

## Installation & Usage (Server)

1. Download the server executable.
2. Run the server (it will create necessary files).
3. Place all files you want to distribute in the ```updates``` folder.
4. Run the server again to serve everything in the updates folder to the clients.

## Installation & Usage (Client)

1. Download the client executable. 
2. Run the client (it will create necessary files). 
3. Configure ```upman.json``` with your settings. 
4. Run the client to receive updates and launch your program.

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
This configuration connects the client to a local server and runs the following command after all files are downloaded:

```bash
java -jar ./explorer.jar
```

## Example
Click on the image to start the video: <br>
[![Watch the example video](http://i3.ytimg.com/vi/xPc7W2W3aHU/hqdefault.jpg)](https://hc-cdn.hel1.your-objectstorage.com/s/v3/eec2cac4be0db51f_updatemanager_-_example.mp4)
