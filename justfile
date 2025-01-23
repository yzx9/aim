# just is a command runner, Justfile is very similar to Makefile, but simpler.

default:
  @just --list

test:
  uv run --with pytest \
    pytest --doctest-modules

test-cov:
  uv run --with pytest --with pytest-cov \
    pytest \
      --doctest-modules \
      --junitxml=junit/test-results.xml \
      --cov=aim --cov-report=xml --cov-report=html
