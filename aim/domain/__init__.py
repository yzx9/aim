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


from aim.domain import organization, project, user
from aim.domain.organization import Organization
from aim.domain.project import Project
from aim.domain.user import User

__all__ = ["Organization", "Project", "User"]


def init(
    *,
    repo_organization: organization.Repository,
    repo_project: project.Repository,
    repo_user: user.Repository,
):
    organization.init(repo_organization)
    project.init(repo_project)
    user.init(repo_user)
