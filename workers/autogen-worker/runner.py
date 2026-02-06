import os

from autogen import AssistantAgent, UserProxyAgent


def run_autogen(prompt: str) -> str:
    model = os.getenv("AUTOGEN_MODEL", "gpt-4o-mini")
    api_key = os.getenv("AUTOGEN_API_KEY", "")
    llm_config = {
        "model": model,
        "api_key": api_key,
    }

    assistant = AssistantAgent(name="assistant", llm_config=llm_config)
    user = UserProxyAgent(name="user", code_execution_config=False)

    user.initiate_chat(assistant, message=prompt)
    return assistant.last_message().get("content", "AutoGen completed task.")


if __name__ == "__main__":
    import sys

    prompt = sys.argv[1] if len(sys.argv) > 1 else "Summarize this task."
    print(run_autogen(prompt))
