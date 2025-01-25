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


import aim.domain.project.config as config
import aim.domain.project.repository as repo
from aim.domain.project.project import Project
from aim.domain.project.repository import Repository
from aim.util import IdGenerator

__all__ = ["Project", "Repository", "init"]


def init(*, repository: Repository, id_generator: IdGenerator[int]):
    repo.init(repository)
    config.init(id_generator)
