PACKAGE_NAME=algorithmia
PACKAGE_VERSION=`grep version ../Cargo.toml | head -1 | cut -d'"' -f2`

ARCH=amd64
TARBALL=algorithmia_linux64.tar.gz
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
	if cd $(OUTDIR)/algorithmia-git; then git pull || true; else git clone ssh://aur@aur.archlinux.org/algorithmia-git.git $(OUTDIR)/algorithmia-git; fi
	sed -e "s/{{VERSION}}/$(SHA_VERSION)/g" arch/algorithmia-git/PKGBUILD.template > $(OUTDIR)/algorithmia-git/PKGBUILD
	cd $(OUTDIR)/algorithmia-git && mksrcinfo

aur-bin: $(OUTDIR) $(TARBALL)
	$(eval MD5SUM=`md5sum $(TARBALL) | cut -f1 -d' '`)
	$(eval VERSION=`grep version ../Cargo.toml | head -1 | cut -d'"' -f2 | sed s/-/_/g`)
	if cd $(OUTDIR)/algorithmia-bin; then git pull || true; else git clone ssh://aur@aur.archlinux.org/algorithmia-bin.git $(OUTDIR)/algorithmia-bin; fi
	sed -e "s/{{VERSION}}/$(VERSION)/g; s/{{MD5SUM}}/$(MD5SUM)/g" arch/algorithmia-bin/PKGBUILD.template > $(OUTDIR)/algorithmia-bin/PKGBUILD
	cd $(OUTDIR)/algorithmia-bin && mksrcinfo

deb: $(OUTDIR) $(TARBALL)
	bundle install
	bundle exec fpm -f -s tar -t deb -v $(PACKAGE_VERSION) -n $(PACKAGE_NAME) -p "$(OUTDIR)/$(PACKAGE_NAME)_$(PACKAGE_VERSION)_$(ARCH).deb" --prefix $(PREFIX) $(TARBALL)
