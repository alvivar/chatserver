## Protocol

### Parameters

-   **`model`**: Specifies the model you'd like to use (e.g. `gpt4`, `gpt3`, `llama2`, etc.)

-   **`main_prompt`**: This is the primary system prompt. It will be sent to the system each time, in conjunction with the memory.

-   **`memory`**: Determines the number of previous chats the system should remember. Acceptable values range from `0` to `n`.

### Configuring the Server

To modify the configuration, send a command to the server in the following format:

```
!<configuration_variable> <desired_value>
```

For example:

```
!model gpt4
!main_prompt You are a robot! You forgot how to be human!
!memory 4
```

**Note**: When `!` precedes a word, the system recognizes it as a configuration command. However, if `!` is placed elsewhere in a message, it's treated as a regular character.

This is also valid:

```
!model gpt4 !main_prompt You are a robot! You forgot how to be human! !memory 4
```
