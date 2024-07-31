#!/bin/bash

set -e

# Install task
TASK_VERSION="v3.35.0"
TASK_PATH=~/.local/bin
TASK_COMPLETION_PATH=~/.bash-completion/completions

if [[ $CI == "1" ]]; then
    TASK_PATH="/usr/local/bin/"
fi

if [[ $(task --version 2>&1 | cut -d ' ' -f 3) != ${TASK_VERSION} ]]; then
    mkdir -p ${TASK_PATH}
    bash -c "$(curl --location https://taskfile.dev/install.sh)" -- -b ${TASK_PATH} -d ${TASK_VERSION}
    cat ~/.bashrc | grep "^PATH=\$PATH:~/.local/bin" || echo "PATH=\$PATH:~/.local/bin" >>~/.bashrc

    mkdir -p ${TASK_COMPLETION_PATH}
    wget -O ${TASK_COMPLETION_PATH}/task.bash \
        https://raw.githubusercontent.com/go-task/task/${TASK_VERSION}/completion/bash/task.bash
    cat ~/.bashrc | grep "source ${TASK_COMPLETION_PATH}/task.bash" ||
        echo "source ${TASK_COMPLETION_PATH}/task.bash" >>~/.bashrc
    echo -e "\nReload shell: source ~/.bashrc"
    if test -f ~/.zshrc; then
        cat ~/.zshrc | grep "^PATH=\$PATH:~/.local/bin" || echo "PATH=\$PATH:~/.local/bin" >>~/.zshrc
        echo -e "\nReload shell: source ~/.zshrc"
    fi
fi
