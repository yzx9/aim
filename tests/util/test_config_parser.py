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

from io import StringIO

import pytest

from aim.util.config_parser import ConfigParser


def test_parse_int():
    parser = ConfigParser(cli_args={"timeout": "30"})
    assert parser.parse_int("timeout") == 30
    assert parser.parse_int("missing", default=10) == 10
    assert parser.parse_int("missing") is None
    assert parser.parse_int("timeout", min_value=40, default=50) == 50
    assert parser.parse_int("timeout", max_value=20, default=15) == 15


def test_parse_float():
    parser = ConfigParser(cli_args={"ratio": "0.75"})
    assert parser.parse_float("ratio") == 0.75
    assert parser.parse_float("missing", default=1.0) == 1.0
    assert parser.parse_float("missing") is None
    assert parser.parse_float("ratio", min_value=1.0, default=2.0) == 2.0


def test_parse_str():
    parser = ConfigParser(cli_args={"mode": "fast"})
    assert parser.parse_str("mode") == "fast"
    assert parser.parse_str("missing", default="slow") == "slow"
    assert parser.parse_str("missing") is None
    assert parser.parse_str("mode", allowed_values=["slow"]) is None


def test_parse_bool():
    parser = ConfigParser(cli_args={"debug": "true"})
    assert parser.parse_bool("debug") is True
    assert parser.parse_bool("missing", default=False) is False
    assert parser.parse_bool("missing") is None
    parser_false = ConfigParser(cli_args={"debug": "false"})
    assert parser_false.parse_bool("debug") is False


@pytest.mark.parametrize(["ports"], [["80,443,8080"], ["80, 443, 8080"]])
def test_parse_ints(ports: str):
    parser = ConfigParser(cli_args={"ports": ports})
    assert parser.parse_ints("ports") == [80, 443, 8080]
    assert parser.parse_ints("missing", default=[3000]) == [3000]
    assert parser.parse_ints("missing") is None
    assert parser.parse_ints("ports", min_value=100) == [443, 8080]
    assert parser.parse_ints("ports", max_value=100) == [80]


@pytest.mark.parametrize(["ratios"], [["0.1,0.5,0.9"], ["0.1, 0.5, 0.9"]])
def test_parse_floats(ratios: str):
    parser = ConfigParser(cli_args={"ratios": ratios})
    assert parser.parse_floats("ratios") == [0.1, 0.5, 0.9]
    assert parser.parse_floats("missing", default=[1.0]) == [1.0]
    assert parser.parse_floats("missing") is None
    assert parser.parse_floats("ratios", min_value=0.5) == [0.5, 0.9]


@pytest.mark.parametrize(["modes"], [["fast,slow"], ["fast, slow"]])
def test_parse_strings(modes: str):
    parser = ConfigParser(cli_args={"modes": modes})
    assert parser.parse_strings("modes") == ["fast", "slow"]
    assert parser.parse_strings("modes", allowed_values=["fast"]) == ["fast"]
    assert parser.parse_strings("missing", default=["medium"]) == ["medium"]
    assert parser.parse_strings("missing") is None


def test_env_variable_parsing(monkeypatch: pytest.MonkeyPatch):
    monkeypatch.setenv("APP_PORT", "5000")
    parser = ConfigParser(env_prefix="APP")
    assert parser.parse_int("port") == 5000


def test_env_file_parsing():
    env_file = StringIO("TEST_KEY=foobar\n")
    parser = ConfigParser(env_files=[env_file])
    assert parser.parse_str("test_key") == "foobar"


def test_cli_and_env_name_style(monkeypatch: pytest.MonkeyPatch):
    monkeypatch.setenv("APP_MAX_TIMEOUT", "60")
    parser = ConfigParser(cli_args={"max-timeout": "30"}, env_prefix="APP")
    assert parser.parse_int("max_timeout") == 30  # CLI args take precedence
    parser_no_cli = ConfigParser(env_prefix="APP")
    assert parser_no_cli.parse_int("max_timeout") == 60  # Env var fallback
