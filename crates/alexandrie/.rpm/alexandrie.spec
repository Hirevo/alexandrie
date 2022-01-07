%define __spec_install_post %{nil}
%define __os_install_post %{_dbpath}/brp-compress
%define debug_package %{nil}

Name: alexandrie
Summary: An alternative crate registry, implemented in Rust.
Version: @@VERSION@@
Release: @@RELEASE@@%{?dist}
License: MIT or ASL 2.0
Group: System Environment/Daemons
Source0: %{name}-%{version}.tar.gz

BuildRoot: %{_tmppath}/%{name}-%{version}-%{release}-root
BuildRequires: systemd

Requires(post): systemd
Requires(preun): systemd
Requires(postun): systemd

%description
%{summary}

%prep
%setup -q

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
cp -a * %{buildroot}
mkdir -p %{buildroot}%{_localstatedir}/db/alexandrie/crate-index
mkdir -p %{buildroot}%{_localstatedir}/db/alexandrie/crate-storage

%clean
rm -rf %{buildroot}

%pre
/usr/bin/getent group alexandrie > /dev/null || /usr/sbin/groupadd -r alexandrie
/usr/bin/getent passwd alexandrie > /dev/null || /usr/sbin/useradd -r -d /usr/bin/alexandrie -s /sbin/nologin -g alexandrie alexandrie

%post
[ ! -d "%{_localstatedir}/db/alexandrie/crate-index/.git" ] && echo "Creating crate-index git repository in %{_localstatedir}/db/alexandrie/crate-index. Please setup origin and configure the crate index in: %{_localstatedir}/db/alexandrie/crate-index/config.json" && git -C %{_localstatedir}/db/alexandrie/crate-index init && git -C %{_localstatedir}/db/alexandrie/crate-index add config.json && git commit -C %{_localstatedir}/db/alexandrie/crate-index -m 'Added `config.json`'
%systemd_post alexandrie.service

%preun
%systemd_preun alexandrie.service

%postun
%systemd_postun_with_restart alexandrie.service

%files
%defattr(-,root,root,-)
%{_bindir}/*
%{_unitdir}/alexandrie.service
%config(noreplace) %{_sysconfdir}/alexandrie.conf
%{_datadir}/alexandrie/assets/*
%{_datadir}/alexandrie/templates/*
%{_datadir}/alexandrie/syntect/dumps/*
%attr(-,alexandrie,alexandrie) %dir %{_localstatedir}/db/alexandrie
%attr(-,alexandrie,alexandrie) %dir %{_localstatedir}/db/alexandrie/crate-index
%config(noreplace) %attr(-,alexandrie,alexandrie) %{_localstatedir}/db/alexandrie/crate-index/config.json
%attr(-,alexandrie,alexandrie) %dir %{_localstatedir}/db/alexandrie/crate-storage
