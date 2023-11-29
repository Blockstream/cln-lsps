#!/usr/bin/env python
import click
import sys
import pandas as pd
from click.exceptions import UsageError
from dataclasses import dataclass
import requests 
import typing as t
import json
import numpy as np

@click.command()
@click.option("file", "--file", help="The json file that will be used to compute the fees")
@click.option("tutorial", "--tutorial", default=False, help="Follow the guided tutorial to estimate your first fee", is_flag=True)
def cli(file, tutorial):
    if tutorial:
        do_tutorial()
    elif file:
        df = pd.read_json(file)
        compute_fee(df)
    else:
        print(f"To get started, try `{sys.argv[0]} --tutorial`")




def do_tutorial():
    print_intro()
    currency = get_currency()
    feerate = get_feerate()
    remind_onchain_costs(feerate, currency)
    df = do_questionnaire(feerate, currency)
    compute_fee(df)
    

def wait_for_enter():
    click.prompt("Press enter to continue...", default=True, value_proc=lambda _ : True, hide_input=True, show_default=False, prompt_suffix="")
    click.echo("")
    click.echo("")

def print_intro():
    click.echo("Welcome to the LSPS1 fee estimation-tool")
    click.echo("")
    click.echo("You want to be an LSP and provide channels using LSPS1")
    click.echo("However, configuring the fees is not easy because some of the parameters are not intuitive")
    click.echo("But, we've got you convered!")
    click.echo("")
    click.echo("We'll give you a couple of examples and you'll provide a reasonable fee.")
    click.echo("Just, use your gut-feeling for now. You can improve your pricing-structure later")
    click.echo("")
    click.echo("Once you have completed this questionnaire. We'll output the approriate fee parameters for you.")
    click.echo("")
    wait_for_enter()

@dataclass
class Currency:
    conversion : float
    unit : str
    format_func : t.Any

    def sats_to_string(self, sat_value : float):
        currency_value = float(sat_value)/self.conversion
        currency_string = self.format_func(currency_value)
        return f"{currency_string} {self.unit}"

    def to_sats(self, currency_calue : float):
        sat_value = float(currency_calue)*self.conversion
        return sat_value


def get_currency():
    click.echo("Please look at the example question below")
    click.echo("> What is a reasonable fee to lease a channel with a capacity of 100_000 sats for 30 days. Please answer in sats?")
    click.echo("")
    click.echo("We'll ask you similar questions later. In this example we used sats as a currency")
    click.echo("What currency would you prefer?")
    click.echo("[1] sat")
    click.echo("[2] BTC")
    click.echo("[3] bit")
    click.echo("[4] other fiat currency")

    def value_proc(prompt_value : str):
        try:
            int_value = int(prompt_value)

            if int_value == 1:
                return Currency(conversion=1.0, unit="sat", format_func = lambda x: f"{x:_.0f}")
            if int_value == 2:
                return Currency(conversion=100_000_000.0, unit="BTC", format_func = lambda x: f"{x:_.8f}")
            if int_value == 3:
                return Currency(conversion=100.0, unit="bit", format_func = lambda x: f"{x:_.2f}")
            if int_value == 4:
                name = click.prompt("What is the name of the fiat-currency you'd like to use?", default="USD")
                value : float = click.prompt(f"What is the current exchange rate. 1 BTC = xxxx {name}", type=float)
                return Currency(conversion=100_000_000.0/value, unit=name, format_func = lambda x: f"{x:_.2f}")
            else:
                raise Exception()
        except Exception as e:
            raise UsageError("Please pick one of the provided options") from e
        
    currency = click.prompt(text=">", prompt_suffix="", value_proc=value_proc)
    click.echo(f"Great! We'll use {currency.unit} with an exchange rate of 1 BTC = {currency.sats_to_string(100_000_000)}");
    wait_for_enter()
    return currency

def get_feerate():
    response = requests.get("https://blockstream.info/api/fee-estimates")
    if response.status_code == 200:
        response = response.json()
        feerate = response["6"] 
        return feerate
    else:
        click.echo("Failed to query feerate. You'll have to provide a feerate manually")
        feerate = click.prompt("Please provide a fee-rate in (sat/vbyte)", type=float)
        return feerate


    
def remind_onchain_costs(feerate, currency):
    click.echo(f"The current feerate = {feerate} sat/vbyte")
    click.echo(f"Estimated channel open costs:  \t{currency.sats_to_string(168.0*feerate)}")
    click.echo(f"Estimated channel close costs: \t{currency.sats_to_string(feerate*288.0)}")
    wait_for_enter()

def do_questionnaire(feerate, currency):

    questionnaire = [
        {
            "capacity_sat" : 100_000, 
            "display_length" : "1 week",
            "expiry_blocks" : 7*24*6,
            "feerate" : feerate
        },
        {
            "capacity_sat" : 5_000_000,
            "display_length" : "1 month",
            "expiry_blocks" : 31*24*6,
            "feerate" : feerate
        },
        {
            "capacity_sat" : 10_000_000,
            "display_length" : "1 day",
            "expiry_blocks" : 24*6,
            "feerate" : feerate
        },
        {
            "capacity_sat" : 300_000,
            "display_length" : "14 days",
            "expiry_blocks" : 14*24,
            "feerate" : feerate
        }
    ]

    for question in questionnaire:
        capacity = question["capacity_sat"]
        length = question["display_length"]
        click.echo(f"A channel of {currency.sats_to_string(capacity)} for {length}")
        reasonable_fee = click.prompt("What is a reasonable fee?", type=float)
        question["reasonable_fee"] = currency.to_sats(reasonable_fee)

    with open("questionaire.json", "w") as fh:
        json.dump(questionnaire, fp=fh)

    click.echo("You've collected some data")
    click.echo("We'll store the data into `questionaire.json` for you")
    df = pd.read_json("questionaire.json")
    click.echo("You can manually edit the file")
    click.echo("If you ever want to recompte the fees based on that file you can do")
    click.echo(f"  {sys.argv[0]} --file questionaire.json")
    return df

def compute_fee(data):
    # We'll do a linear regression
    # A + onchain_cost + x * capacity_sat*expiry_blocks*B = reasonable_fee_rate
    # with y = reasonable_fee - onchain_cost
    # with x = capacity_sat*expiry_block
    #
    # we get y = A + x*B 

    print("Estimating the fee using least-squares regression")

    y = data["reasonable_fee"] - (168+288)*data["feerate"]
    x = data["capacity_sat"]*data["expiry_blocks"]

    A = np.column_stack((np.ones(np.size(x)), x))
    b = np.array(y).T

    x, _, _, _ = np.linalg.lstsq(A,b,rcond=None)

    base_fee = x[0]
    ppb = x[1]

    print(f"lsps1_fee_computation_base_base_sat=  {base_fee:10_.0f} sat")
    print(f"lsps1_fee_computation_onchain_ppm=    {ppb*1e9:10_.0f}")
    print(f"lsps1_fee_computation_liquidity_ppb=  {1_000_000:10_.0f}")


if __name__ == "__main__":
    cli()


