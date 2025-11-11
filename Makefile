.PHONY: install uninstall

install: $(HOME)/.local/bin/orbit

uninstall:
	unlink "$(HOME)/.local/bin/orbit"

$(HOME)/.local/bin/orbit: orbit
	mkdir --parents "$(dir $@)"
	ln --symbolic "$(abspath $<)" "$@"
