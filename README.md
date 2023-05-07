# PCLI Sync

This is a helper tool for automating the data replication with Physna. It works in conjunction with [PCLI](https://github.com/jchultarsky101/pcli).

Some use cases require that we synchronize the contents of a local directory with a Physna folder. For example, 
we may have a list of 3D models in a directory on our disk and we want to automatically upload to Physna any new files as
they are saved to our local directory and also remove models from Physna as files are deleted in the local directory.

This is a command line tool. Use it in a terminal session or you could wrap it as Windows service or Linux daemon to work
in the background unattended.

## Change Log

The current version is 0.1.0.

##### Version 0.1.0

Initial pre-production version.

## Dependencies

PCLI Sync uses PCLI for the actual Physna operations. Make sure that you have PCLI installed and configured correctly first. Ensure that PCLI is
added to your system path so that you can execute by simply typing **pcli** without having to specify the path to the executable. For example:

````bash
pcli help
````

## Installation

PCLI Sync is available for download as an OS-navive executable. You can find the version specific to your operating system under releases. Downlooad it to your computer in
a directory of your choice and configure your system PATH appropriatelly.

There is no configuration file. All input paramters are provided as command line arguments when you start the process.

## Usage

PCLI Sync offers in-line help. To see details for all arguments, execute the program with **--help** as the sole argument:

````bash
pcli-sync --help
  _____   _____ _      _____    _____
 |  __ \ / ____| |    |_   _|  / ____|
 | |__) | |    | |      | |   | (___  _   _ _ __   ___
 |  ___/| |    | |      | |    \___ \| | | | '_ \ / __|
 | |    | |____| |____ _| |_   ____) | |_| | | | | (__
 |_|     \_____|______|_____| |_____/ \__, |_| |_|\___|
                                       __/ |
                                      |___/

Version 0.1.0
jchultarsky@physna.com

Physna file sync. Monitors a directory for changes and synchronizes the contents with Physna.

Usage: pcli-sync --directory <DIRECTORY> --tenant <TENANT> --folder-id <FOLDER_ID> --units <UNITS>

Options:
  -d, --directory <DIRECTORY>
          Directory to monitor for changes

  -t, --tenant <TENANT>
          Physna tenant

  -f, --folder-id <FOLDER_ID>
          Physna folder ID

  -u, --units <UNITS>
          Unit of measure

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
````

This will print out a help screen.

Here we have:

* **--directory** - the local path of a directory on your computer you want to monitor for changes
* **--tenant** - your Physna tenant ID. See the PCLI documentation or your Physna representative for details
* **--folder-id** - your Physna folder ID. See the PCLI documenation for details
* **--units** - the unit of measure that will be used during data uploads. For example "mm" or "in"

### Start the program

Here is a full example on how to start the process:

````bash
pcli-sync --tenant=mytenant --folder-id=100 --directory=./test --units=mm
  _____   _____ _      _____    _____
 |  __ \ / ____| |    |_   _|  / ____|
 | |__) | |    | |      | |   | (___  _   _ _ __   ___
 |  ___/| |    | |      | |    \___ \| | | | '_ \ / __|
 | |    | |____| |____ _| |_   ____) | |_| | | | | (__
 |_|     \_____|______|_____| |_____/ \__, |_| |_|\___|
                                       __/ |
                                      |___/

Version 0.1.0
jchultarsky@physna.com

Watching directory ./test... To exit, press Ctrl-C.
````

In the example above:

* our tenant ID as provided by Physna is "mytenant"
* our Physna folder ID is 100
* our local directory we want to monitor for changes is ./test
* we want files to be uploaded as unit of measure in millimeters ("mm")

### Operations

Once started, **pcli-sync** will continously execute until stopped. Changes in directory contents are detected withing few seconds.

#### New file detected

When a new file is stored in the source directory, it will be uploaded to Physna.

#### File deleted

If a file is deleted from the local directory, **pcli-sync** will issue a delete command to Physna.

#### File is modified

If the data in a file is changes, **pcli-sync** will first delete the model from Physna and then re-upload the new version to Physna.

### Stop the program

To stop the program, you can navigate to the terminal session in which it executes, and simply enter Ctrl-C. Alternativelly, you can kill the process or 
if you are running it as a service, you can stop the service.

There are no side effects of terminating the program. It does not make any changes on your local dist. Incomplete operations with Physna will be automatically cancelled on the backend.

## Support

This is an open source project. For support, create an issue in GitHub. If you require direct assistance, please e-mail to jchultarsky@physna.com.
