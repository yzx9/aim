# Copyright 2025 Zexin Yuan
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#    http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.


import pathlib

import click

from app import App

PROJECT_ROOT = pathlib.Path(__file__).parent.parent


@click.group()
def cli(): ...


@cli.command()
@click.option("--dev", is_flag=True, help="Enable development")
@click.option(
    "--env-prefix", type=str, default="AIM", help="Environment variable prefix"
)
def serve(dev: bool, env_prefix: str):
    """Start the AIM web application"""
    click.echo("🚦Initializing AIM.")
    app = App(str(PROJECT_ROOT.absolute()), dev=dev, env_prefix=env_prefix)

    click.echo(f"🚀Starting server on {app.config.host}:{app.config.port}.")
    app.run()

    click.echo("🛑Server stopped.")


if __name__ == "__main__":
    cli()
