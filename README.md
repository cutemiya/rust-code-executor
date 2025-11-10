# Code executor
### A docker container is created to execute the code for security reasons.
### Before using it, run the following command in the console:
```shell
  docker pull python:3.9-slim
  docker pull node:18-alpine  
  docker pull golang:1.19-alpine
  docker pull kotlin:latest
```
## Endpoints:

[`GET`] `/health` - healthcheck.

> `RS-payload`
>```json
>{
>  "status": "healthy"
>}

[`POST`] `/execute` - execute your code.

> `RQ-paylaod`
>```json
>{
>  "code": "print('Hello world')",
>  "language": "Python",
>  "timeout": 20
>}
>```
>>`language` - enum it can take the following values: `Python`, `Golang`, `JavaScript`, `Kotlin`
>>
>>`code` - code :)
>>
>>`timeout` - optional field that is measured in seconds
>>
>>`stdin` - optional field, input-values
>
> `RS-payload`
>```json
>{
>  "execution_id": "9efa0030-e248-4483-b77c-a020c3924601",
>  "stdout": "",
>  "stderr": "",
>  "exit_code": 0,
>  "duration": 10.179345678,
>  "timed_out": false
>}
>```
>>`execution_id` - uuid
>>
>>`stdout` - output
>>
>>`stderr` - error output
>>
>>`exit_code` - program exit code
>>
>>`dutation` - program execution time
>>
>>`timet_out` - flag that determines whether there was a timeout or not

[`POST`] `/execute/file` - execute your code from file, using multipart.
> `multipart`
>
>>`code` - ur code file
>
> `query-params`
>>`language` - enum it can take the following values: `Python`, `Golang`, `JavaScript`, `Kotlin`
>>
>>`timeout` - optional field that is measured in seconds
>>
>>`stdin` - optional field, input-values
>
> `RS-payload`
>```json
>{
>  "execution_id": "9efa0030-e248-4483-b77c-a020c3924601",
>  "stdout": "",
>  "stderr": "",
>  "exit_code": 0,
>  "duration": 10.179345678,
>  "timed_out": false
>}
>```
>>`execution_id` - uuid
>>
>>`stdout` - output
>>
>>`stderr` - error output
>>
>>`exit_code` - program exit code
>>
>>`dutation` - program execution time
>>
>>`timet_out` - flag that determines whether there was a timeout or not
