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


import aim.domain as domain
import aim.infrastructure.rdbms as rdbms
from aim.config import Config
from aim.domain import Organization, Project, User

__all__ = ["Config", "Organization", "Project", "User", "init"]


def init(config: Config):
    """Initialize all application domains with their repositories."""

    # Initialize repository instances
    org_repo = rdbms.OrganizationRepository()
    project_repo = rdbms.ProjectRepository()
    user_repo = rdbms.UserRepository()

    # Initialize domains
    domain.init(
        repo_organization=org_repo,
        repo_project=project_repo,
        repo_user=user_repo,
        machine_id=config.machine_id,
    )
