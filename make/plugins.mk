# Modified by ACRYL, 2026
# Skills-only ACRYL microVMs distribution; no runtime build is required.
PLUGIN_COVERAGE_PYTHON ?= $(PLUGIN_PYTHON)
PLUGIN_COVERAGE = PYTHONDONTWRITEBYTECODE=1 COVERAGE_FILE=target/coverage/plugins/.coverage COVERAGE_RCFILE=scripts/plugins/.coveragerc $(PLUGIN_COVERAGE_PYTHON) -m coverage


plugin\:acrylmicrovm\:check: plugin-acrylmicrovm-force
	@$(PLUGIN_PYTHON) scripts/plugins/acrylmicrovm.py check
	@$(MAKE) --no-print-directory test:unit:plugins

plugin\:acrylmicrovm\:dist: plugin-acrylmicrovm-force plugin\:acrylmicrovm\:check
	@$(PLUGIN_PYTHON) scripts/plugins/acrylmicrovm.py dist

plugin\:acrylmicrovm\:check\:cc: plugin-acrylmicrovm-force plugin\:acrylmicrovm\:dist
	@claude plugin validate --strict plugins/acrylmicrovm
	@claude plugin validate --strict target/plugins/acrylmicrovm-marketplace
plugin\:acrylmicrovm\:coverage: plugin-acrylmicrovm-force
	@mkdir -p target/coverage/plugins
	@$(PLUGIN_COVERAGE) run -m unittest discover -s tests/plugins -p 'test_*.py'
	@$(PLUGIN_COVERAGE) run --append scripts/plugins/acrylmicrovm.py check
	@$(PLUGIN_COVERAGE) run --append scripts/plugins/acrylmicrovm.py dist
	@$(PLUGIN_COVERAGE) xml -o target/coverage/plugins/coverage.xml
	@$(PLUGIN_COVERAGE) report -m
