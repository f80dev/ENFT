#cd /home/root/dev/temp/adder/interaction
#pip install erdpy==1.0.18
#source devnet.snippets.sh && deploy

PROXY=https://devnet-gateway.elrond.com
CHAINID="D"
ALICE="/home/root/dev/PEM/alice.pem"
ADDRESS=$(erdpy data load --key=address-devnet)
DEPLOY_TRANSACTION=$(erdpy data load --key=deployTransaction-devnet)

deploy() {
    erdpy contract build
    erdpy --verbose contract deploy --proxy=${PROXY} --chain=${CHAINID} --project=. --recall-nonce --pem=${ALICE} --gas-limit=150000000 --send --outfile="deploy-devnet.interaction.json"

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