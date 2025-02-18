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


import dataclasses
from pathlib import Path
from typing import Self, TextIO

__all__ = ["Config"]


PathOrTextIO = str | Path | TextIO


@dataclasses.dataclass
class Config:
    jwt_secret: str
    exp_access_token: int
    exp_refresh_token: int

    @classmethod
    def new(cls, cert_sign: PathOrTextIO) -> Self:
        # TODO: watch cert file
        if isinstance(cert_sign, str):
            with open(cert_sign, "r") as f:
                jwt_secret = f.read()
        elif isinstance(cert_sign, Path):
            jwt_secret = cert_sign.read_text()
        else:
            jwt_secret = cert_sign.read()

        return cls(
            jwt_secret=jwt_secret,
            exp_access_token=5 * 60,  # 5 min
            exp_refresh_token=30 * 24 * 60 * 60,  # 30 days
        )
