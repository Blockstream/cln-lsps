
# This private functions sets all cli-arguments
function _load_env() {
	export GIT_ROOT=$(git rev-parse --show-toplevel)

	export CLIENT_LIGHTNING_DIR=/tmp/lsps_c1
	export SERVER_LIGHTNING_DIR=/tmp/lsps_s1
	export CLIENT_LIGHTNING_CONFIG=$CLIENT_LIGHTNING_DIR/config
	export SERVER_LIGHTNING_CONFIG=$SERVER_LIGHTNING_DIR/config

	export LIGHTNING_CLI=$(which lightning-cli)
	export LIGHTNINGD=$(which lightningd)
	export BITCOIN_CLI="$(which bitcoin-cli) -regtest"
	export BITCOIND="$(which bitcoind) -regtest"

	echo "lightning-cli = $LIGHTNING_CLI"
	echo "lightningd = $LIGHTNINGD"
	echo "bitcoin-cli = $BITCOIN_CLI"
	echo "bitcoind = $BITCOIND"
	echo ""


	alias bitcoin-cli="bitcoin-cli -regtest"
	alias bitcoind="bitcoind -regtest -daemon"

	alias s1-lightningd="$LIGHTNINGD --lightning-dir=$SERVER_LIGHTNING_DIR --daemon"
	alias s1-cli="$LIGHTNING_CLI --lightning-dir=$SERVER_LIGHTNING_DIR"
	alias s1-log="less $SERVER_LIGHTNING_DIR/log"
	alias s1-restart="s1-cli stop & s1-lightningd"

	alias c1-lightningd="$LIGHTNINGD --lightning-dir=$CLIENT_LIGHTNING_DIR --daemon"
	alias c1-cli="$LIGHTNING_CLI --lightning-dir=$CLIENT_LIGHTNING_DIR"
	alias c1-log="less $CLIENT_LIGHTNING_DIR/log"
	alias c1-restart="c1-cli stop & c1-lightningd"

	echo "--------------------------------------------------------"
	echo "This script created aliases for an LSP-server (named s1)"
	echo "and an lsp-client named c1"
	echo ""
	echo "It exposes the following utilites for each node."
	echo "The examples here are for s1. But you can also use c1-cli"
	echo "- s1-cli           lightning-cli for s1"
	echo "- s1-lightningd    lightningd for s1"
	echo "- s1-log           See the logs for s1"
	echo "- s1-restart       Restart the node s1"
	echo ""
	echo "You can also use"
	echo "- connect          To connect all nodes"
	echo "- stop             To stop all nodes"
	echo "- start            To start all nodes"
	echo ""
	echo "For developers it is useful to stop, make and "
	echo "start to test changes in your plugin"
	echo "-------------------------------------------------------"

	function connect() {
		export SERVER_ID=$(s1-cli getinfo | jq .id)
		export SERVER_HOST=$(s1-cli getinfo | jq .binding[0].address)
		export SERVER_PORT=$(s1-cli getinfo | jq .binding[0].port)

		$LIGHTNING_CLI --lightning-dir=$CLIENT_LIGHTNING_DIR connect $SERVER_ID $SERVER_HOST $SERVER_PORT

	}

	function stop_nodes() {
		$LIGHTNING_CLI --lightning-dir=$CLIENT_LIGHTNING_DIR stop
		$LIGHTNING_CLI --lightning-dir=$SERVER_LIGHTNING_DIR stop
	}

	function start_nodes() {
		echo "Starting LSP-clients"
		echo "- Start c1"
		$LIGHTNINGD --lightning-dir=$CLIENT_LIGHTNING_DIR --daemon
		echo "Starting LSP-servers"
		echo "- Start s1"
		$LIGHTNINGD --lightning-dir=$SERVER_LIGHTNING_DIR --daemon
	}

	# fund c1 0.2 
	# 
	# Fund node with 0.2 btc
	function fund_node() {
		node=$1
		amount=$2
		address=$($LIGHTNING_CLI --lightning-dir=/tmp/lsps_$node newaddr | jq -r .bech32)
		echo "Funding $2 BTC to $1 on $address"
		$BITCOIN_CLI sendtoaddress $address $amount
		$BITCOIN_CLI -generate 7
	}

	function open_channel() {
		FROM_NODE=$1
		TO_NODE=$2
		AMOUNT=$3

		FROM_ID=$($LIGHTNING_CLI --lightning-dir=/tmp/lsps_$FROM_NODE getinfo | jq -r .id)
		TO_ID=$($LIGHTNING_CLI --lightning-dir=/tmp/lsps_$TO_NODE getinfo | jq -r .id)

		echo "Funding channel from $FROM_NODE to $TO_NODE of $AMOUNT sat"
		$LIGHTNING_CLI --lightning-dir=/tmp/lsps_$FROM_NODE fundchannel $TO_ID $AMOUNT
		$BITCOIN_CLI -generate 7
	}

	function load_ids() {
		export S1_ID=$(s1-cli getinfo | jq .id)
		export C1_ID=$(c1-cli getinfo | jq .id)

		echo "Set S1_ID=$S1_ID"
		echo "Set C1_ID=$C1_ID"
	}
	function delete_nodes() {
		rm -rf $CLIENT_LIGHTNING_DIR
		rm -rf $SERVER_LIGHTNIGN_DIR
	}
}

function load_env() {
	export GIT_ROOT=$(git rev-parse --show-toplevel)
	export CLIENT_LIGHTNING_DIR=/tmp/lsps_c1
	export SERVER_LIGHTNING_DIR=/tmp/lsps_s1

	if test ! -d $CLIENT_LIGHTNING_DIR; then
		echo "No configuration for client yet. Try running set_env first"
	fi

	if test ! -d $SERVER_LIGHTNING_DIR; then
		echo "No configuration for server yet. Try running set_env first"
	fi

	_load_env

	load_ids

}

function set_env() {
	_load_env

	if test -d $CLIENT_LIGHTNING_DIR; then
  		echo "The lightning-dir for client already exists"
		echo "We'll stop lightningd if it runs and create a new and clean folder"
		$LIGHTNING_CLI --lightning-dir=$CLIENT_LIGHTNING_DIR stop &> /dev/null
		rm -rf $CLIENT_LIGHTNING_DIR
	fi

	if test -d $SERVER_LIGHTNING_DIR; then
  		echo "The lightning-dir for server already exists"
		echo "We'll stop lightningd if it runs and create a new and clean folder"
		$LIGHTNING_CLI --lightning-dir=$SERVER_LIGHTNING_DIR stop &> /dev/null
		rm -rf $SERVER_LIGHTNING_DIR
	fi

	mkdir -p $CLIENT_LIGHTNING_DIR
	mkdir -p $SERVER_LIGHTNING_DIR

	# Configuring the LSP-server
	echo "regtest" > $SERVER_LIGHTNING_CONFIG
	echo "disable-plugin=clnrest" >> $SERVER_LIGHTNING_CONFIG
	echo "daemon" >> $SERVER_LIGHTNING_CONFIG
	echo "log-level=DEBUG" >> $SERVER_LIGHTNING_CONFIG
	echo "log-file=$SERVER_LIGHTNING_DIR/log" >> $SERVER_LIGHTNING_CONFIG
	echo "plugin=$GIT_ROOT/build/plugins/lsps0-server/lsps0-server" >> $SERVER_LIGHTNING_CONFIG
	echo "addr=localhost:20202" >> $SERVER_LIGHTNING_CONFIG
	echo "alias=LSP-server" >> $SERVER_LIGHTNING_CONFIG
	echo "lsps1_enable=true" >> $SERVER_LIGHTNING_CONFIG
	echo "lsps1_min_initial_client_balance_sat=0" >> $SERVER_LIGHTNING_CONFIG
	echo "lsps1_max_initial_client_balance_sat=0" >> $SERVER_LIGHTNING_CONFIG
	echo "lsps1_min_initial_lsp_balance_sat=0" >> $SERVER_LIGHTNING_CONFIG
	echo "lsps1_max_initial_lsp_balance_sat=100000000" >> $SERVER_LIGHTNING_CONFIG
	echo "lsps1_min_channel_balance_sat=0" >> $SERVER_LIGHTNING_CONFIG
	echo "lsps1_max_channel_balance_sat=100000000" >> $SERVER_LIGHTNING_CONFIG

	# Configuring the LSP-client
	echo "regtest" > $CLIENT_LIGHTNING_CONFIG
	echo "disable-plugin=clnrest" >> $CLIENT_LIGHTNING_CONFIG
	echo "daemon" >> $CLIENT_LIGHTNING_CONFIG
	echo "log-file=$CLIENT_LIGHTNING_DIR/log" >> $CLIENT_LIGHTNING_CONFIG
	echo "plugin=$GIT_ROOT/build/plugins/lsps0-client/lsps0-client" >> $CLIENT_LIGHTNING_CONFIG
	echo "plugin=$GIT_ROOT/test/plugins/accept_channel_slowly.py" >> $CLIENT_LIGHTNING_CONFIG
	echo "addr=localhost:20401" >> $CLIENT_LIGHTNING_CONFIG
	echo "alias=LSP-client" >> $CLIENT_LIGHTNING_CONFIG
	echo "Starting LSP-client"
	$LIGHTNINGD --lightning-dir=$CLIENT_LIGHTNING_DIR --daemon
	echo "Starting LSP-server"
	$LIGHTNINGD --lightning-dir=$SERVER_LIGHTNING_DIR --daemon

	load_ids
	connect
	fund_node s1 10.0
	fund_node c1 0.1

        while ! "$LIGHTNING_CLI" -F --lightning-dir="/tmp/lsps_c1" listfunds | grep -q "outputs"
        	do
                        sleep 1
                done

        while ! "$LIGHTNING_CLI" -F --lightning-dir="/tmp/lsps_s1" listfunds | grep -q "outputs"
        	do
                        sleep 1
                done

	open_channel c1 s1 99999
}

function stop_all() {
	c1-cli stop
	s1-cli stop
}