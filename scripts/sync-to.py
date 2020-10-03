#!/usr/bin/env python3
import click
from subprocess import Popen, PIPE

# Syncs the current project to the remote server
@click.command()
@click.option('-u', '--user', "user", help='The remote user', required=True)
@click.option('-d', '--dest', "dest", help='The remote ssh server address', required=True)
@click.option('-p', '--path', "path", help='The remote path', required=True)
@click.option('--port', "port", default=22, help='The remote ssh server port.')
def sync(user, dest, path, port):
    pwdout, _ = Popen(["pwd"], stdout=PIPE).communicate() # TODO: Do smarter detection
    project_dir = pwdout.decode().strip()
    project_src = project_dir + "/src"
    project_toml = project_dir + "/Cargo.toml"
    cmd_dir = ["scp", '-P', str(port), "-r", project_src, "{user}@{dest}:{path}".format(user=user, dest=dest, path=path)]
    run(cmd_dir)
    print("Copied src folder to {dest}".format(dest=dest))
    cmd_toml = ["scp", '-P', str(port), project_toml, "{user}@{dest}:{path}".format(user=user, dest=dest, path=path)]
    run(cmd_toml)
    print("Copied Cargo.toml to {dest}".format(dest=dest))

def run(cmd):
    scpout, scperr = Popen(cmd, stdout=PIPE, stderr=PIPE).communicate()
    # print(scpout.decode().strip())
    # print(scperr.decode().strip())
    

if __name__ == "__main__":
    sync()
