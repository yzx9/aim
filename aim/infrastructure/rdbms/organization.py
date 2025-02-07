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


from typing import Optional

import sqlalchemy as sa
from sqlalchemy.orm import mapped_column

from aim.domain.organization import Organization
from aim.infrastructure.rdbms._base import Base, BaseMixin, BaseRepositoryPlus
from aim.util import AsyncSessionHandler

__all__ = ["OrganizationModel", "OrganizationRepository"]


class OrganizationModel(BaseMixin, Base):
    """SQLAlchemy model representing the organization table."""

    __tablename__ = "organizations"

    name = mapped_column(sa.String(64), nullable=False)


class OrganizationRepository(BaseRepositoryPlus[Organization, OrganizationModel]):
    def __init__(self, session_handler: AsyncSessionHandler) -> None:
        super().__init__(session_handler, OrganizationModel)

    def _to_model(
        self, entity: Organization, model: Optional[OrganizationModel] = None
    ) -> OrganizationModel:
        if model is None:
            model = OrganizationModel()

        model.id = entity.id
        model.name = entity.name
        return model

    def _to_entity(self, model: OrganizationModel) -> Organization:
        return Organization(id=model.id, name=model.name, repository=self)
