#cd /home/root/dev/temp/adder/interaction
#source devnet.snippets.sh && deploy

ALICE="/home/root/dev/PEM/alice.pem"
ADDRESS=$(erdpy data load --key=address-devnet)
DEPLOY_TRANSACTION=$(erdpy data load --key=deployTransaction-devnet)
PROXY=https://devnet-gateway.elrond.com
CHAINID="D"

deploy() {
    cd ..
    erdpy contract build
    cd interaction
    erdpy --verbose contract deploy --chain=${CHAINID} --proxy=${PROXY} --project=.. --recall-nonce --pem=${ALICE} --gas-limit=50000000 --arguments 0 --send --outfile="deploy-devnet.interaction.json" || return

    TRANSACTION=$(erdpy data parse --file="deploy-devnet.interaction.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(erdpy data parse --file="deploy-devnet.interaction.json" --expression="data['emitted_tx']['address']")

    erdpy data store --key=address-devnet --value=${ADDRESS}
    erdpy data store --key=deployTransaction-devnet --value=${TRANSACTION}

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

add() {
    read -p "Enter number: " NUMBER
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=50000000 --function="add" --arguments ${NUMBER} --send
}

getSum() {
    erdpy --verbose contract query ${ADDRESS} --function="getSum"
}