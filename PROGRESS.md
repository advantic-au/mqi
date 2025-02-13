
MQ verbs
========

Refer to <https://www.ibm.com/docs/en/ibm-mq/latest?topic=calls-call-descriptions>

| Verb    | Description                         | Wrapped  | API | Comments |
|---------|-------------------------------------|----------|----------|-|
| MQBACK  | Back out changes                    | ✔ | ✔ |  |
| MQBEGIN | Begin unit of work                  | ✔ | Experiment | |
| MQBUFMH | Convert buffer into message handle  | ✔ |   |  |
| MQCB    | Manage callback                     | ✔ | Experiment | |
| MQCLOSE | Close object                        | ✔ | ✔ |  |
| MQCMIT  | Commit changes                      | ✔ | ✔ | |
| MQCONN  | Connect queue manager               | ✔ | Not Used | Equivalent features in MCONNX |
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
| MQSTAT  | Retrieve status information         | ✔ | ✔ |  |
| MQSUB   | Register subscription               | ✔ | ✔ |  |
| MQSUBRQ | Subscription request                | ✔ |   |  |
| MQXCNVC | Convert characters                  | ✔ | ✔ |  |

MQAI functions
==============

Refer to <https://www.ibm.com/docs/en/ibm-mq/latest?topic=reference-mqai-calls>

| Function                  | Description                                                            | Wrapped | API | Comments |
|---------------------------|------------------------------------------------------------------------|---|---|---|
| mqCreateBag               | Create a new bag                                                       | ✔ | ✔ |   |
| mqClearBag                | Delete all user items from the bag                                     | ✔ | ✔ |   |
| mqDeleteBag               | Delete the specified bag                                               | ✔ | ✔ | On Drop |
| mqGetBag                  | Remove a message from the specified queue as bag data                  | ✔ | ✔ |   |
| mqPutBag                  | Convert the contents of the specified bag into a PCF message and send  | ✔ | ✔ |   |
| mqTruncateBag             | Reduce the number of user items in a user bag to the specified value   | ✔ | ✔ |   |
| mqAddInquiry              | Add a selector to an administration bag                                | ✔ | ✔ |   |
| mqDeleteItem              | Remove one or more user items from a bag                               | ✔ | ✔ |   |
| mqAddInteger              | Add an integer item identified by a user selector                      | ✔ | ✔ |   |
| mqAddIntegerFilter        | Add an integer filter identified by a user selector                    | ✔ | ✔ |   |
| mqAddInteger64            | Add a 64-bit integer item identified by a user selector                | ✔ | ✔ |   |
| mqAddString               | Add a character data item identified by a user selector                | ✔ | ✔ |   |
| mqAddStringFilter         | Add a string filter identified by a user selector                      | ✔ | ✔ |   |
| mqAddByteString           | Add a byte string identified by a user selector                        | ✔ | ✔ |   |
| mqAddByteStringFilter     | Add a byte string filter identified by a user selector                 | ✔ | ✔ |   |
| mqSetInteger              | Modify an integer item that is present in the bag                      | ✔ | ✔ |   |
| mqSetIntegerFilter        | Modify an integer filter item that is present in the bag               | ✔ | ✔ |   |
| mqSetInteger64            | Modify a 64-bit integer item that is present in the bag                | ✔ | ✔ |   |
| mqAddBag                  | Nest a bag in another bag                                              | ✔ | ✔ |   |
| mqSetString               | Modify a character data item that is present in the bag                | ✔ | ✔ |   |
| mqSetStringFilter         | Modify a string filter item that is present in the bag                 | ✔ | ✔ |   |
| mqSetByteString           | Modify a byte string data item that is present in the bag              | ✔ | ✔ |   |
| mqSetByteStringFilter     | Modify a byte string filter item that is present in the bag            | ✔ | ✔ |   |
| mqInquireInteger          | Request the value of an integer data item that is present in the bag   | ✔ | ✔ |   |
| mqInquireIntegerFilter    | Request the value and operator of an integer filter item               | ✔ | ✔ |   |
| mqInquireInteger64        | Request the value of a 64-bit integer data item                        | ✔ | ✔ |   |
| mqInquireByteString       | Requests the value of a byte string data item                          | ✔ | ✔ |   |
| mqInquireString           | Request the value of a character data item                             | ✔ | ✔ |   |
| mqInquireStringFilter     | Request the value and operator of a string filter item                 | ✔ | ✔ |   |
| mqInquireByteStringFilter | Request the value and operator of a byte string filter item            | ✔ | ✔ |   |
| mqInquireBag              | Inquire the value of a bag handle                                      | ✔ | ✔ |   |
| mqCountItems              | Return the number of occurrences of items                              | ✔ | ✔ |   |
| mqExecute                 | Send an administration command message and wait for the reply          | ✔ | ✔ |   |
| mqBagToBuffer             | Convert the bag into a PCF message in the supplied buffer              | ✔ | ✔ |   |
| mqBufferToBag             | Convert the supplied buffer into bag form                              | ✔ | ✔ |   |
| mqInquireItemInfo         | Return information about a specified item in a bag                     | ✔ | ✔ |   |
| mqTrim                    | Trim the blanks from a blank-padded string, then terminates it with a null | ✘ |   | Can be trivially implemented in safe rust |
| mqPad                     | Pad a null-terminated string with blanks                               | ✘  |   | Can be trivially implemented in safe rust |

Examples
========

| Name              | Description                        | Status |
|-------------------|------------------------------------|--------|
| Amqsbo            | Handling a poison message - MQGMO_SYNCPOINT | |
| amqscb            | Callback handling instead of MQGET | |
| amqsconn          | Connect to remote QM              | |
| amqsconntls       | TLS connection                    | |
| amqsdlh           | put and get message with DLH      | |
| amqsgbr           | browse loop                       | |
| amqsget           | get loop                          | |
| amqsinq           | inquire queue attributes          | |
| amqsjwt           | token authentication              | |
| amqspcf           | equivalent to DISPLAY Q(x) ALL.   | |
| amqsprop          | manipulate message properties (put and get) | |
| amqspub           | publish to a topic                | |
| amqsset           | set queue attributes (MQSET)      | |
| amqssub           | subscribe (managed, non-durable)  | |
| rust async        | | |
| handling headers. | | |
| Request Reply (provider) | | |
| request/reply (consumer) | | |
| additional attributes on connect / get / put | | |
| Function for a parameter | | |
