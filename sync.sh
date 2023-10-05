#!/bin/bash
# Convenience script for syncing folder to an EC2 instance
# Useful for limited testing in remote environments

# Check if the number of arguments provided is correct
if [ "$#" -ne 2 ]; then
    echo "Usage: $0 <path-to-ssh-key> <ec2-ip-address>"
    exit 1
fi

# Extract the arguments into variables
SSH_KEY="$1"
EC2_IP="$2"


rsync -avz -e "ssh -i $SSH_KEY" --exclude-from='./exclusions.txt' . ec2-user@$EC2_IP:~/tonk-services