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


import datetime

from aim.domain import organization, project, user
from aim.domain.organization import Organization
from aim.domain.project import Project
from aim.domain.user import User
from aim.util.id_generator import SnowflakeGenerator

__all__ = ["Organization", "Project", "User"]

# Epoch for Snowflake IDs, default: 2025-01-01 00:00:00 UTC in milliseconds
_EPOCH = int(
    (datetime.datetime(2025, 1, 1) - datetime.datetime(1970, 1, 1)).total_seconds()
    * 1000
)


def init(
    *,
    repo_organization: organization.Repository,
    repo_project: project.Repository,
    repo_user: user.Repository,
    machine_id: int,
):
    def _new_id_generator():
        return SnowflakeGenerator(machine_id, epoch=_EPOCH)

    organization.init(repository=repo_organization, id_generator=_new_id_generator())
    project.init(repository=repo_project, id_generator=_new_id_generator())
    user.init(repository=repo_user, id_generator=_new_id_generator())
