#!/bin/bash
set -e

rm -r /home/jonas/.config/Code/User/globalStorage/botloader.botloader-vscode/typings/*
cp -r typings/* /home/jonas/.config/Code/User/globalStorage/botloader.botloader-vscode/typings