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


import aim.infrastructure.rdbms as rdbms
from aim.domain import init as init_domains

__all__ = ["init"]


def init(machine_id: int):
    """Initialize all application domains with their repositories."""

    # Initialize repository instances
    org_repo = rdbms.OrganizationRepository()
    project_repo = rdbms.ProjectRepository()
    user_repo = rdbms.UserRepository()

    # Initialize domains
    init_domains(
        repo_organization=org_repo,
        repo_project=project_repo,
        repo_user=user_repo,
        machine_id=machine_id,
    )
