import json
import os
# 读取修正后的JSON文件
with open('master.config', 'r') as file:
    config_data = json.load(file)

# 处理JSON数据并生成所需的数据结构
result_list = [(item['master'], item['slave']) for item in config_data]

for master_port, slave_ports in result_list:
    # 启动master任务
    master_command = f'cargo run --bin master {master_port} {" ".join(map(str, slave_ports))} &'
    os.system(master_command)
    # print (master_command)
    # subprocess.Popen(master_command, shell=True)
    
    # 启动slave任务
    for slave_port in slave_ports:
        slave_command = f'cargo run --bin slave {slave_port} &'
        # subprocess.Popen(slave_command, shell=True)
        os.system(slave_command)

