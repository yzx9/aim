# just is a command runner, Justfile is very similar to Makefile, but simpler.

default:
  @just --list

dev:
  cd web && pnpm dev

serve:
  uv run python -m aim serve

test:
  uv run --with pytest \
    pytest --doctest-modules
  cd web && pnpm run test --run

test-cov:
  uv run --with pytest --with pytest-cov \
    pytest \
      --doctest-modules \
      --cov=aim --cov=app --cov-report=xml --cov-report=html

clean:
  rm -rf `find . -name __pycache__`
  find . -type f -name '*.py[co]'  -delete
  find . -type f -name '*~'  -delete
  find . -type f -name '.*~'  -delete
  find . -type f -name '@*'  -delete
  find . -type f -name '#*#'  -delete
  find . -type f -name '*.orig'  -delete
  find . -type f -name '*.rej'  -delete
  rm -f .coverage
  rm -rf coverage
  rm -rf build
  rm -rf htmlcov
  rm -rf dist
