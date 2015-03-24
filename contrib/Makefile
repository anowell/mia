PACKAGE_NAME=algorithmia
PACKAGE_VERSION=0.4.0

ARCH=amd64
TARBALL=algo-linux.tar.gz
DOWNLOAD=https://github.com/algorithmiaio/algorithmia-cli/releases/download/v$(PACKAGE_VERSION)/$(TARBALL)
TMPDIR=tmp
OUTDIR=$(PWD)/out
PREFIX=/usr/bin

default: aur-bin aur-git deb

clean:
	rm -fr $(OUTDIR) || true
	rm -fr $(TMPDIR) || true
	rm -f *.tar.gz

$(TARBALL):
	curl -L -O "$(DOWNLOAD)"

$(OUTDIR):
	mkdir -p $(OUTDIR)

aur-git: $(OUTDIR)
	$(eval SHA_VERSION=`git log -1 --format="%cd.g%h" --date=short | sed 's/-/./g'`)
	mkdir -p $(TMPDIR)/algorithmia-git
	sed -e "s/{{VERSION}}/$(SHA_VERSION)/g" arch/algorithmia-git/PKGBUILD.template > $(TMPDIR)/algorithmia-git/PKGBUILD
	cd $(TMPDIR)/algorithmia-git && SRCPKGDEST=$(OUTDIR) makepkg -fS

aur-bin: $(OUTDIR) $(TARBALL)
	$(eval MD5SUM=`md5sum $(TARBALL) | cut -f1 -d' '`)
	mkdir -p $(TMPDIR)/algorithmia-bin
	sed -e "s/{{VERSION}}/$(PACKAGE_VERSION)/g; s/{{MD5SUM}}/$(MD5SUM)/g" arch/algorithmia-bin/PKGBUILD.template > $(TMPDIR)/algorithmia-bin/PKGBUILD
	cd $(TMPDIR)/algorithmia-bin && SRCPKGDEST=$(OUTDIR) makepkg -fS

deb: $(OUTDIR) $(TARBALL)
	bundle install
	bundle exec fpm -s tar -t deb -v $(PACKAGE_VERSION) -n $(PACKAGE_NAME) -p "$(OUTDIR)/$(PACKAGE_NAME)_$(PACKAGE_VERSION)_$(ARCH).deb" --prefix $(PREFIX) $(TARBALL)
