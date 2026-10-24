Examples
========

| Name                                      | Description                              | Demonstrates     |
|-------------------------------------------|------------------------------------------|------------------|
| [subscribe_managed](subscribe_managed.rs) | Subscribe and get messages from a topic  | Managed subscription and get_message loop |
| [open_put](open_put.rs)                   | Open & put to queue or topic             | Putting a message to a queue or topic |
| [readme](readme.rs)                       | Example for [README.md](../README.md)    | Simple connect, put_message and disconnect |
| [forward](forward.rs)                     | Forwards a single message between queues | Syncpoint, get_data_with, put_message, context, message properties |
| [dlopen2](dlopen2.rs)                     | Load the MQ library at runtime           | Decoupling compile with the MQ libraries, runtime loading |
| [ping_server](ping_server.rs)             | Simple server that echos message to reply queue | Server Request/Reply pattern, MQMD get and set |
| [request_reply](request_reply.rs)         | Send a request to a queue and wait for ar reply | Client Request/Reply pattern, dynamic queues |
