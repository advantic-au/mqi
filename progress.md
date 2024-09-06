
MQ verbs
========

Refer to <https://www.ibm.com/docs/en/ibm-mq/latest?topic=calls-call-descriptions>

| Verb    | Description                         | Wrapped  | API | Comments |
|---------|-------------------------------------|----------|----------|-|
| MQBACK  | Back out changes                    | ✔ |   |  |
| MQBEGIN | Begin unit of work                  | ✔ | Experement | |
| MQBUFMH | Convert buffer into message handle  | ✔ |   |  |
| MQCB    | Manage callback                     | ✔ | Experiment | |
| MQCLOSE | Close object                        | ✔ | ✔ |  |
| MQCMIT  | Commit changes                      | ✔ | Experiment | |
| MQCONN  | Connect queue manager               | ✔ | Not Used | All features in MCONNX |
| MQCONNX | Connect queue manager (extended)    | ✔ | ✔ |  |
| MQCRTMH | Create message handle               | ✔ | ✔ |  |
| MQCTL   | Control callbacks                   | ✔ | Experiment | |
| MQDISC  | Disconnect queue manager            | ✔ | ✔ |  |
| MQDLTMH | Delete message handle               | ✔ | ✔ |  |
| MQDLTMP | Delete message property             | ✔ | ✔ |  |
| MQGET   | Get message                         | ✔ | ✔ |  |
| MQINQ   | Inquire object attributes           | ✔ | ✔ |  |
| MQINQMP | Inquire message property            | ✔ | ✔ |  |
| MQMHBUF | Convert message handle into buffer  | ✔ |   |  |
| MQOPEN  | Open object                         | ✔ | ✔ |  |
| MQPUT   | Put message                         | ✔ | ✔ |  |
| MQPUT1  | Put one message                     | ✔ | ✔ |  |
| MQSET   | Set object attributes               | ✔ | ✔ |  |
| MQSETMP | Set message property                | ✔ | ✔ |  |
| MQSTAT  | Retrieve status information         | ✔ |   |  |
| MQMHBUF | Convert message handle into buffer  | ✔ |   |  |
| MQSUB   | Register subscription               | ✔ | ✔ |  |
| MQSUBRQ | Subscription request                | ✔ |   |  |
| MQXCNVC | Convert characters                  | ✔ |   |  |

Examples
========

| Name              | Description                       | Status |
|-------------------|-----------------------------------|--------|
| subscribe_managed | Perform a subscription to a topic |   ✔    |
| Amqsbo            | Handling a poison message - MQGMO_SYNCPOINT | |
| amqscb            | Callback handling instead of MQGET | |
| amqsconn          |  Connect to remote QM             | |
| amqsconntls       | TLS connection                    | |
| amqsdlh           | put and get message with DLH      | |
| amqsgbr           | browse loop                       | |
| amqsget           | get loop                          | |
| amqsinq           | inquire queue attributes          | |
| amqsjwt           | token authentication              | |
| amqspcf           | equivalent to DISPLAY Q(x) ALL.   | |
| amqsprop          | manipulate message properties (put and get) | |
| amqspub           | publish to a topic                | |
| amqsput           | put to queue                      | |
| amqsset           | set queue attributes (MQSET)      | |
| amqssub           | subscribe (managed, non-durable)  | |
| rust async        | | |
| handling headers. | | |
| Request Reply (provider) | | |
| request/reply (consumer) | | |
| additional attributes on connect / get / put | | |
| Function for a parameter | | |
