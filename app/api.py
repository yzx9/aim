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


from aiohttp import web

from aim.domain import Organization

__all__ = ["setup_apis"]


def setup_apis(r: web.UrlDispatcher) -> None:
    r.add_get("/api/ping", _ping, name="get_ping")
    r.add_post("/api/ping", _ping, name="post_ping")

    r.add_post("/api/organizations", _post_organizations, name="post_organizations")
    r.add_get("/api/organizations/{id}", _get_organization, name="get_organization")


async def _ping(request: web.Request) -> web.Response:
    return web.json_response("pong")


async def _post_organizations(request: web.Request) -> web.Response:
    """Create a new organization.

    Example request body:
    {
        "name": "My Organization"
    }
    """
    data = await request.json()
    name = data["name"]
    if not name:
        raise web.HTTPBadRequest(reason="Organization name cannot be empty")

    organization = await Organization.new(name)
    return web.json_response(
        {"id": organization._id, "name": organization._name}, status=201
    )


async def _get_organization(request: web.Request) -> web.Response:
    """Get an organization by ID."""
    org_id = int(request.match_info["id"])
    organization = await Organization.find(org_id)
    if not organization:
        raise web.HTTPNotFound()

    return web.json_response({"id": organization._id, "name": organization._name})
