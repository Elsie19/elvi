readonly bar="1"
bar="2"
# comment
baz='boo\tnotab';oof=`ls /`
boogle="bar is: '${bar}'"
dbg boogle
dbg oof
hash -r
dir="/usr/share"
cd "${dir}"
if [ -f "/etc/resolv.conf" ]; then
    echo "${PWD}"
    echo "${HOME}"
else
    echo "${PATH}"
    dbg PATH
fi
