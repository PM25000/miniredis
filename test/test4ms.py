import os
import sys
os.system("cd ..")
os.system("cargo run --bin master 8080 8081 &")
os.system("cargo run --bin slave 8081 8080 &")
# sys.sleep(3)
# os.system("cargo run --bin client4ms_tst 8080 8081 &")
# import subprocess
# import os

# # 定义要执行的命令列表
# commands = [
#     "cargo run --bin master 8080 8081",  # 第一个终端执行的命令
#     "cargo run --bin slave 8081 8080",  # 第二个终端执行的命令
#     "cargo run --bin client4ms_tst 8080 8081",       # 第三个终端执行的命令
#     # "echo 'Hello, World!'",  # 第四个终端执行的命令
# ]

# # 创建一个列表来存储终端进程对象
# terminal_processes = []

# # 启动终端并执行命令
# for cmd in commands:
#     terminal = subprocess.Popen(
#         ['gnome-terminal', '--', 'bash', '-c', cmd],
#         stdout=subprocess.PIPE,
#         stderr=subprocess.PIPE 
#     )

# # # 等待所有终端进程完成
# # for terminal in terminal_processes:
# #     terminal.wait()