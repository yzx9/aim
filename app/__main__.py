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


import click
from aiohttp import web

from aim import init
from app import new_app


def run(host: str, port: int, machine_id: int) -> None:
    click.echo("🚦Initializing application.")
    # TODO: Load config
    init(machine_id)

    click.echo(f"🚀Starting server on {host}:{port}.")
    app = new_app()
    web.run_app(app, host=host, port=port)

    click.echo("🛑Server stopped.")


@click.group()
def cli():
    pass


@cli.command()
@click.option("--host", default="127.0.0.1", help="Host to bind to")
@click.option("--port", default=8080, help="Port to listen on")
@click.option("--machine-id", default=0, help="Unique ID for this machine (0-1023)")
def serve(host: str, port: int, machine_id: int):
    """Start the AIM web application"""

    run(host, port, machine_id)


if __name__ == "__main__":
    cli()
