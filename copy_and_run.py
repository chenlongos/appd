#!/usr/bin/env python3
"""
ArceOS 复制和运行脚本 - 用于 Phytium Pi 平台

此脚本自动化以下流程：
1. 通过 SCP 将构建好的 ArceOS 二进制文件复制到目标设备
2. 通过 SSH 重启目标设备
3. 通过串口连接拦截 U-Boot 自动启动序列
4. 直接执行 U-Boot 原生命令序列启动 ArceOS：
   - ext4load mmc 0:1 0x90100000 /home/{用户名}/arceos.bin
   - dcache flush
   - go 0x90100000

重要说明：此脚本假设已配置免密码 sudo 和 SSH 密钥登录。

通用性改进：
此版本已优化为通用脚本，支持任意用户名：
- 目标路径会根据 SSH_USER 自动调整为 /home/{用户名}/arceos.bin
- 直接使用 U-Boot 原生命令，无需依赖预设的环境变量
- 不修改 U-Boot 环境变量，保持系统配置不变

配置变量说明：
脚本中的配置变量（SSH_USER、SSH_HOST、SERIAL_PORT 等）必须根据您的实际环境进行修改，
详细配置说明请参见脚本中的配置变量部分。

必需的前置条件：

0. 硬件和网络准备（必须）：
   - Phytium Pi 设备已正常启动并运行 Linux 系统
   - Phytium Pi 已连接网络且可以通过 SSH 访问
   - Phytium Pi 的串口已正确连接到本机（通过 USB-TTL 转换器等）
   - 确认串口设备路径（如 /dev/ttyUSB0）并配置到脚本中

1. SSH 密钥登录设置（必须）：
   在目标设备上设置 SSH 公钥认证，避免每次都输入密码：
   - 生成密钥对：ssh-keygen -t rsa -b 4096
   - 复制公钥到目标设备：ssh-copy-id username@target_ip
   - 确保可以无密码登录：ssh username@target_ip

2. 免密码 sudo 重启设置（必须）：
   在目标设备上配置：
   - sudo visudo
   - 添加：username ALL=(ALL) NOPASSWD: /usr/sbin/reboot, /sbin/reboot
   （将 'username' 替换为实际用户名）

   如果使用 root 用户运行，请使用：--user root --no-sudo

依赖要求：
- pyserial: pip install pyserial
"""

import subprocess
import time
import sys
import argparse
import logging
from pathlib import Path

try:
    import serial
except ImportError:
    print("Error: pyserial library is not installed.")
    print("Please install it with: pip install pyserial")
    sys.exit(1)

# 配置变量 - 使用前必须根据您的环境正确修改这些参数
# IMPORTANT: 这些配置变量必须根据实际环境进行修改，否则脚本将无法正常工作！

SSH_USER = "user"  # SSH 登录用户名（必须修改为实际用户名）
SSH_HOST = "192.168.0.107"  # 目标设备 IP 地址（必须修改为实际设备 IP）
USE_SUDO = True  # 如果用户需要 sudo 权限重启则设为 True（根据实际情况修改）
SERIAL_PORT = "/dev/ttyUSB0"  # 串口设备路径（必须修改为实际串口设备）
SERIAL_BAUDRATE = 115200  # 串口波特率（通常保持 115200，除非设备要求不同）
ARCEOS_BINARY_PATH = "examples/helloworld/helloworld_aarch64-phytium-pi.bin"  # ArceOS 二进制文件路径（根据实际构建路径修改）

# 动态生成目标路径，自动适配用户名（无需手动修改）
TARGET_PATH = f"/home/{SSH_USER}/arceos.bin"  # 目标设备上的存储路径，会根据SSH_USER自动调整

# Timing configurations
AUTOBOOT_TIMEOUT = 60  # seconds to wait for autoboot message (increased for slow boot)
REBOOT_WAIT_TIME = 1   # seconds to wait after reboot command

# Network connectivity test configurations
PING_COUNT = 10        # number of ping packets to send
PING_INTERVAL = 0.2    # interval between pings (seconds)
PING_TIMEOUT = 60      # maximum time to wait for network connectivity (seconds)

class ArceOSDeployer:
    def __init__(self, ssh_user, ssh_host, serial_port, baudrate, use_sudo=True, 
                 autoboot_timeout=AUTOBOOT_TIMEOUT, ping_timeout=PING_TIMEOUT, 
                 ping_interval=PING_INTERVAL, skip_ping=False):
        self.ssh_user = ssh_user
        self.ssh_host = ssh_host
        self.serial_port = serial_port
        self.baudrate = baudrate
        self.use_sudo = use_sudo
        self.autoboot_timeout = autoboot_timeout
        self.ping_timeout = ping_timeout
        self.ping_interval = ping_interval
        self.skip_ping = skip_ping
        self.serial_conn = None
        
        # Setup logging
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(levelname)s - %(message)s'
        )
        self.logger = logging.getLogger(__name__)

    def wait_for_network(self):
        """Wait for network connectivity using high-frequency ping"""
        self.logger.info(f"Testing network connectivity to {self.ssh_host}")
        self.logger.info(f"Pinging every {self.ping_interval}s for up to {self.ping_timeout}s")
        
        start_time = time.time()
        consecutive_success = 0
        required_consecutive = 3  # Need 3 consecutive successful pings
        
        while time.time() - start_time < self.ping_timeout:
            # Send a single ping with short timeout
            ping_cmd = [
                "ping", 
                "-c", "1",           # Send 1 packet
                "-W", "1",           # Wait 1 second for response
                "-q",                # Quiet output
                self.ssh_host
            ]
            
            try:
                result = subprocess.run(ping_cmd, capture_output=True, text=True, timeout=2)
                if result.returncode == 0:
                    consecutive_success += 1
                    self.logger.info(f"Ping successful ({consecutive_success}/{required_consecutive})")
                    
                    if consecutive_success >= required_consecutive:
                        self.logger.info("Network connectivity confirmed!")
                        return True
                else:
                    if consecutive_success > 0:
                        self.logger.warning("Ping failed, resetting counter")
                    consecutive_success = 0
                    
            except subprocess.TimeoutExpired:
                self.logger.warning("Ping timeout")
                consecutive_success = 0
            except Exception as e:
                self.logger.warning(f"Ping error: {e}")
                consecutive_success = 0
            
            time.sleep(self.ping_interval)
        
        self.logger.error(f"Network connectivity test failed after {self.ping_timeout} seconds")
        return False

    def copy_binary(self, local_path, remote_path):
        """Copy ArceOS binary to target device via SCP"""
        self.logger.info(f"Copying {local_path} to {self.ssh_user}@{self.ssh_host}:{remote_path}")
        
        if not Path(local_path).exists():
            raise FileNotFoundError(f"Local binary not found: {local_path}")
        
        scp_cmd = [
            "scp",
            local_path,
            f"{self.ssh_user}@{self.ssh_host}:{remote_path}"
        ]
        
        try:
            subprocess.run(scp_cmd, check=True, capture_output=True, text=True)
            self.logger.info("Binary copied successfully")
        except subprocess.CalledProcessError as e:
            self.logger.error(f"SCP failed: {e.stderr}")
            raise

    def reboot_target(self):
        """Reboot the target device via SSH"""
        self.logger.info(f"Rebooting target device {self.ssh_host}")
        
        # Choose reboot command based on user privileges
        if self.use_sudo:
            reboot_cmd = "sudo reboot"
            self.logger.info("Using sudo for reboot command")
        else:
            reboot_cmd = "reboot"
            self.logger.info("Using direct reboot command (assuming root privileges)")
        
        ssh_cmd = [
            "ssh",
            "-o", "StrictHostKeyChecking=no",  # Skip host key checking for automation
            "-t",  # Force pseudo-terminal allocation for sudo
            f"{self.ssh_user}@{self.ssh_host}",
            reboot_cmd
        ]
        
        try:
            # Note: This command will likely "fail" as the connection drops during reboot
            result = subprocess.run(ssh_cmd, capture_output=True, timeout=15, text=True, input='\n')
            
            # Check for common sudo password prompts
            if result.returncode != 0 and ("password" in result.stderr.lower() or "password" in result.stdout.lower()):
                self.logger.error("Sudo password required but not provided.")
                self.logger.error("Please set up passwordless sudo for reboot command on target device:")
                self.logger.error(f"  sudo visudo")
                self.logger.error(f"  Add: {self.ssh_user} ALL=(ALL) NOPASSWD: /usr/sbin/reboot, /sbin/reboot")
                raise RuntimeError("Reboot failed: sudo password required")
            
        except subprocess.TimeoutExpired:
            # Expected behavior during reboot - connection drops
            self.logger.info("SSH connection dropped (expected during reboot)")
        except subprocess.CalledProcessError as e:
            # Also expected during reboot
            self.logger.info(f"SSH command exited with code {e.returncode} (expected during reboot)")
        
        self.logger.info("Reboot command sent, serial port will capture boot sequence")
        # Minimal wait to ensure reboot command is processed
        time.sleep(REBOOT_WAIT_TIME)

    def connect_serial(self):
        """Establish serial connection"""
        self.logger.info(f"Connecting to serial port {self.serial_port}")
        
        try:
            self.serial_conn = serial.Serial(
                port=self.serial_port,
                baudrate=self.baudrate,
                timeout=1,
                parity=serial.PARITY_NONE,
                stopbits=serial.STOPBITS_ONE,
                bytesize=serial.EIGHTBITS
            )
            self.logger.info("Serial connection established")
        except serial.SerialException as e:
            self.logger.error(f"Failed to connect to serial port: {e}")
            raise

    def wait_for_autoboot(self):
        """Wait for U-Boot autoboot message and interrupt it"""
        self.logger.info("Waiting for U-Boot autoboot message...")
        
        start_time = time.time()
        buffer = ""
        
        while time.time() - start_time < self.autoboot_timeout:
            if self.serial_conn.in_waiting > 0:
                data = self.serial_conn.read(self.serial_conn.in_waiting).decode('utf-8', errors='ignore')
                buffer += data
                print(data, end='', flush=True)  # Show boot output to user
                
                # Check for autoboot message
                if "Hit any key to stop autoboot" in buffer:
                    self.logger.info("Autoboot message detected, sending interrupt...")
                    # Send space character to interrupt autoboot
                    self.serial_conn.write(b' ')
                    self.serial_conn.flush()
                    time.sleep(0.5)
                    return True
            
            time.sleep(0.1)
        
        self.logger.error("Timeout waiting for autoboot message")
        return False

    def wait_for_uboot_prompt(self):
        """Wait for U-Boot prompt"""
        self.logger.info("Waiting for U-Boot prompt...")
        
        start_time = time.time()
        buffer = ""
        
        while time.time() - start_time < 30:  # 30 second timeout
            if self.serial_conn.in_waiting > 0:
                data = self.serial_conn.read(self.serial_conn.in_waiting).decode('utf-8', errors='ignore')
                buffer += data
                print(data, end='', flush=True)
                
                # Look for U-Boot prompt patterns
                current_line = buffer.split('\n')[-1]
                if any(prompt in current_line for prompt in ["# ", "=> ", "Phytium-Pi#", "phytium# "]):
                    self.logger.info(f"U-Boot prompt detected: '{current_line.strip()}'")
                    time.sleep(0.5)
                    return True
            
            time.sleep(0.1)
        
        self.logger.error("Timeout waiting for U-Boot prompt")
        return False

    def execute_boot_command(self):
        """Execute ArceOS boot commands in a single command sequence using U-Boot native commands"""
        self.logger.info("Executing ArceOS boot sequence using U-Boot native commands...")
        
        # Execute all commands in one sequence separated by semicolons
        boot_sequence = f"ext4load mmc 0:1 0x90100000 {self.target_path}; dcache flush; go 0x90100000\r\n"
        self.logger.info(f"Executing boot sequence: {boot_sequence.strip()}")
        
        # Send the combined command sequence
        self.serial_conn.write(boot_sequence.encode('utf-8'))
        self.serial_conn.flush()
        
        self.logger.info("ArceOS boot sequence initiated, monitoring output...")
        
        # Monitor output for a longer period since commands execute sequentially
        start_time = time.time()
        while time.time() - start_time < 60:  # Monitor for 60 seconds
            if self.serial_conn.in_waiting > 0:
                data = self.serial_conn.read(self.serial_conn.in_waiting).decode('utf-8', errors='ignore')
                print(data, end='', flush=True)
                
                # Check for load errors in the output
                if "Error" in data or "error" in data:
                    self.logger.error("Error detected in U-Boot output")
                    # Continue monitoring but log the error
                    
            time.sleep(0.1)

    def cleanup(self):
        """Close serial connection"""
        if self.serial_conn and self.serial_conn.is_open:
            self.serial_conn.close()
            self.logger.info("Serial connection closed")

    def deploy(self, binary_path, target_path):
        """Complete deployment workflow"""
        try:
            # Step 1: Wait for network connectivity (if not skipped)
            if not self.skip_ping:
                if not self.wait_for_network():
                    raise RuntimeError("Failed to establish network connectivity")
            else:
                self.logger.info("Skipping network connectivity test")
            
            # Step 2: Copy binary
            self.copy_binary(binary_path, target_path)
            
            # Step 3: Connect to serial first (before reboot)
            self.connect_serial()
            
            # Step 4: Reboot target (serial is now ready to capture boot)
            self.reboot_target()
            
            # Step 5: Wait for and interrupt autoboot
            if not self.wait_for_autoboot():
                raise RuntimeError("Failed to interrupt autoboot")
            
            # Step 6: Wait for U-Boot prompt
            if not self.wait_for_uboot_prompt():
                raise RuntimeError("Failed to get U-Boot prompt")
            
            # Step 7: Execute ArceOS boot commands directly
            self.execute_boot_command()
            
        except Exception as e:
            self.logger.error(f"Deployment failed: {e}")
            raise
        finally:
            self.cleanup()


def main():
    parser = argparse.ArgumentParser(description='Deploy and run ArceOS on Phytium Pi')
    parser.add_argument('--binary', '-b', default=ARCEOS_BINARY_PATH,
                       help='Path to ArceOS binary file')
    parser.add_argument('--target', '-t', default=TARGET_PATH,
                       help='Target path on remote device')
    parser.add_argument('--host', default=SSH_HOST,
                       help='SSH host address')
    parser.add_argument('--user', '-u', default=SSH_USER,
                       help='SSH username')
    parser.add_argument('--serial', '-s', default=SERIAL_PORT,
                       help='Serial port device')
    parser.add_argument('--baudrate', '-B', type=int, default=SERIAL_BAUDRATE,
                       help='Serial baudrate')
    parser.add_argument('--no-sudo', action='store_true',
                       help='Do not use sudo for reboot (assume root privileges)')
    parser.add_argument('--autoboot-timeout', type=int, default=AUTOBOOT_TIMEOUT,
                       help=f'Timeout for waiting autoboot message (default: {AUTOBOOT_TIMEOUT}s)')
    parser.add_argument('--ping-timeout', type=int, default=PING_TIMEOUT,
                       help=f'Timeout for network connectivity test (default: {PING_TIMEOUT}s)')
    parser.add_argument('--ping-interval', type=float, default=PING_INTERVAL,
                       help=f'Interval between ping tests (default: {PING_INTERVAL}s)')
    parser.add_argument('--skip-ping', action='store_true',
                       help='Skip network connectivity test before SCP')
    
    args = parser.parse_args()
    
    print("=" * 60)
    print("ArceOS Deployment Script for Phytium Pi")
    print("=" * 60)
    print(f"Binary: {args.binary}")
    print(f"Target: {args.user}@{args.host}:{args.target}")
    print(f"Serial: {args.serial} @ {args.baudrate}")
    print("=" * 60)
    
    use_sudo = not args.no_sudo  # Invert the flag logic
    deployer = ArceOSDeployer(
        args.user, args.host, args.serial, args.baudrate, use_sudo, 
        args.autoboot_timeout, args.ping_timeout, args.ping_interval, args.skip_ping
    )
    
    try:
        deployer.deploy(args.binary, args.target)
        print("\n" + "=" * 60)
        print("Deployment completed successfully!")
        print("ArceOS should now be running on the target device.")
        print("=" * 60)
    except KeyboardInterrupt:
        print("\nDeployment interrupted by user")
        deployer.cleanup()
        sys.exit(1)
    except Exception as e:
        print(f"\nDeployment failed: {e}")
        deployer.cleanup()
        sys.exit(1)


if __name__ == "__main__":
    main()