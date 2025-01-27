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


from typing import Self

from aim.domain.project.config import generate_id


class Project:
    def __init__(self, id: int, organization_id: int, name: str):
        self.id = id
        self.organization_id = organization_id
        self.name = name

    async def save(self):
        raise NotImplementedError()

    @classmethod
    async def new(cls, organization_id: int, name: str) -> Self:
        id = await generate_id()
        return cls(id, organization_id, name)

    @classmethod
    async def find(cls, id: int) -> Self:
        raise NotImplementedError()
