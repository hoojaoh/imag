ECHO=$(shell which echo) -e
RUBY=$(shell which ruby)
BUNDLE=$(shell which bundle)

all:
	@$(ECHO) "There is no default target here"

bundle:
	@$(ECHO) "[BUNDLE]"
	@$(BUNDLE) install --path vendor/bundle

tarball: bundle
	@$(ECHO) "[RAKE  ][thermite]: tarball"
	@CARGO_TARGET=debug $(BUNDLE) exec rake thermite:tarball

test: tarball
	@$(ECHO) "[RAKE  ][thermite]: test"
	@CARGO_TARGET=debug $(BUNDLE) exec rake test

.FORCE:

.PHONY: all

