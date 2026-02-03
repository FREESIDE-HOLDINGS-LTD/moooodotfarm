.PHONY: all
all:
	rsync -az --delete --exclude 'target/' --progress ./ server:/home/filip/server/docker/moooodotfarm/repo/
	ssh -t server 'cd /home/filip/server/docker/moooodotfarm && make update'
