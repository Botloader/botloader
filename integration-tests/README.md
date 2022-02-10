This is a set of scripts that are run as "tests". Each script once completed successfully needs to call `sendScriptCompletion` from `lib.ts` to report the test success. In the case of failure you can just throw an exception and it will be caught by the system.

It's pretty much just built around a log subscriber and `console.log` so these tests are as close as you can get to real world usage as you can get with automated testing.

The goal is to have every runtime function be tested by this system.

## Running tests

Note that this will **DROP A DATABASE** NAMED "botloader-integration-testing".

Running the "run.sh" script will run them, assuming it actually works for you, you have postgres and rust installed etc...

You can use the "-filter regex-pattern" flag to only run a subset of tests.

There's still a lot of tests missing as this system was added a bit recently, and improvements still needs to be done as a whole in a lot of places.