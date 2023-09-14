import json
import os
import subprocess
import signal
# 读取修正后的JSON文件
with open('master.config', 'r') as file:
    config_data = json.load(file)

# 处理JSON数据并生成所需的数据结构
result_list = [(item['master'], item['slave']) for item in config_data]

process_dict = {}

for master_port, slave_ports in result_list:
    # 启动master任务
    master_command = f'start cargo run --bin master {master_port} {" ".join(map(str, slave_ports))}'
    master_process = subprocess.Popen(master_command, shell=True)
    master_pid = master_process.pid
    process_dict[master_port] = master_pid
    # print (master_command)
    # subprocess.Popen(master_command, shell=True)
    
    # 启动slave任务
    for slave_port in slave_ports:
        slave_command = f'start cargo run --bin slave {slave_port}'
        # subprocess.Popen(slave_command, shell=True)
        slave_process = subprocess.Popen(slave_command, shell=True)
        slav_pid = slave_process.pid
        process_dict[slave_port] = slav_pid
        os.system(slave_command)



proxy_command = f'start cargo run --bin proxy'
os.system(proxy_command)

client_command = f'cargo run --bin client'
os.system(client_command)

set_command1 = f'set 1 1'
os.system(set_command1)

set_command2 = f'set 2 2'
os.system(set_command2)

for pid in process_dict.values():
    os.kill(pid, )
