readonly bar="1"
# bar="2"
# comment
baz='boo\tnotab';oof=`ls -l /`
boogle="bar is: '${bar}'"
hash -r
dir="/usr/share"
cd "${dir}"
if [ -f "/etc/resolv.conf" ]; then
    echo "\$PWD is: ${PWD}"
    echo "\$HOME is: ${HOME}"
    echo "Contents of running 'ls /' is ${oof}"
else
    echo "${PATH}"
    dbg PATH
fi

for i in "foo" "bar" "baz"; do
    echo "${i}"
done

echo "My PID is $$ and current process is $0"

for i in ~/Projects/*; do
    echo "${i}"
done

echo "${PATH}"

foo() {
    echo "bar"
}

echo "Count: $#: ${1}, ${2}, ${3}"
