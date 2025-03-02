.PHONY: help install uninstall

help:
	@awk -F: '/^[a-zA-Z0-9._-]+:/ && !/\.PHONY/ {printf "%s, ", $$1}' Makefile | sed 's/, $$/\n/'

install: $(HOME)/.local/bin/orbit
	@echo >&2 "Successfully installed: $@"

uninstall: $(HOME)/.local/bin/orbit
	@rm "$^"
	@echo >&2 "Successfully uninstalled: $^"

$(HOME)/.local/bin/orbit: $(PWD)/orbit
	@grep -G ":$(HOME)/.local/bin:" <(echo ":$(PATH):") > /dev/null
	@mkdir --parents $(dir $@) && ln --symbolic "$<" "$@"
