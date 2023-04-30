
# Funbot 
A QQ bot based on [go-cqhttp](https://github.com/Mrs4s/go-cqhttp)


A simple GPT-3.5 chatbot based on [gpt-3.5](https://platform.openai.com/) API;

You can use it in QQ group by @ it or in private chat by sending a message to it.

Bot’s QQ:2834957564
## Commands

### /gpt reset

Clear the current conversation history,and reset system prompt to default.

### /gpt role <prompt>

Set the system prompt to <prompt>.

### /gpt4 <msg>

Chat with GPT-4.

## Build from source
```bash
git clone <repo>

cd <repo>

cargo build --release
```

