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


import aim.domain.user.config as config
import aim.domain.user.repository as repo
from aim.domain.user.repository import Repository
from aim.domain.user.user import User
from aim.util.id_generator import IdGenerator

__all__ = ["User", "Repository", "init"]


def init(*, repository: Repository, id_generator: IdGenerator[int]):
    config.init(id_generator)
    repo.init(repository)
