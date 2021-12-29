import { useCallback, useEffect, useRef, useState } from 'react';
import Head from 'next/head';
import Image from 'next/image';
import Link from 'next/link';
import Crossword from 'react-crossword-near';
import { parseSeedPhrase, generateSeedPhrase } from 'near-seed-phrase';

import * as nearAPI from 'near-api-js';
import { createGridData, loadGuesses } from 'react-crossword-near/dist/es/util';

import {
    mungeBlockchainCrossword,
    parseSolutionSeedPhrase,
    viewMethodOnContract,
    walletConnection,
    isWindow,
    b64toUtf8,
} from '_utils';
import dataHardCode from '_utils/hardcoded_data';
import getConfig from '_config';

import NoCrosswordsPage from 'components/NoCrosswordsPage';
import WonPage from 'components/WonPage';
import SuccessPage from 'components/SuccessPage';

const nearConfig = getConfig(process.env.NEXT_PUBLIC_NEAR_ENV);
const Home = () => {
    const [solvedPuzzle, setSolvedPuzzle] = useState((isWindow && localStorage.getItem('playerSolvedPuzzle')) || null);
    const [showLoader, setShowLoader] = useState(false);
    const [currentUser, setCurrentUser] = useState(false);
    const [claimError, setClaimError] = useState('');
    const [data, setData] = useState(dataHardCode);
    const [needsNewAccount, setNeedsNewAccount] = useState(false);

    const crosswordSolutionPublicKey = isWindow && localStorage.getItem('crosswordSolutionPublicKey');
    const playerKeyPair = isWindow && JSON.parse(localStorage.getItem('playerKeyPair'));

    useEffect(async () => {
        const existingKey = localStorage.getItem('playerKeyPair');
        if (!existingKey) {
            // Create a random key in here
            const seedPhrase = generateSeedPhrase();
            localStorage.setItem('playerKeyPair', JSON.stringify(seedPhrase));
        }

        const walConn = await walletConnection();
        if (walConn.getAccountId()) {
            setCurrentUser(walConn.getAccountId());
        }

        const chainData = await viewMethodOnContract(nearConfig, 'get_unsolved_puzzles');
        if (chainData.puzzles.length) {
            const data = mungeBlockchainCrossword(chainData.puzzles);
            localStorage.setItem('crosswordSolutionPublicKey', chainData.puzzles[0]['solution_public_key']);
            setData(data);
        }
    }, []);

    const claimPrize = async e => {
        e.preventDefault();
        const winer_account_id = document.getElementById('claim-account-id').value.toLowerCase();
        const memo = document.getElementById('claim-memo').value;
        const keyStore = new nearAPI.keyStores.InMemoryKeyStore();
        const keyPair = nearAPI.utils.key_pair.KeyPair.fromString(playerKeyPair.secretKey);
        await keyStore.setKey(nearConfig.networkId, nearConfig.contractName, keyPair);
        nearConfig.keyStore = keyStore;

        const near = await nearAPI.connect(nearConfig);
        const crosswordAccount = await near.account(nearConfig.contractName);

        let transaction;
        try {
            setShowLoader(true);
            if (needsNewAccount) {
                transaction = await crosswordAccount.functionCall({
                    contractId: nearConfig.contractName,
                    methodName: 'claim_reward_new_account',
                    args: {
                        crossword_pk: solvedPuzzle,
                        new_acc_id: winner_account_id,
                        new_pk: playerKeyPair.publicKey,
                        memo,
                    },
                    gas: '300000000000000', // You may omit this for default gas
                    attachedDeposit: 0, // You may also omit this for no deposit
                });
            } else {
                transaction = await crosswordAccount.functionCall({
                    contractId: nearConfig.contractName,
                    methodName: 'claim_reward',
                    args: {
                        crossword_pk: solvedPuzzle,
                        receiver_acc_id: winner_account_id,
                        memo,
                    },
                    gas: '300000000000000', // You may omit this for default gas
                    attachedDeposit: 0, // You may also omit this for no deposit
                });
                console.log('transaction: ', transaction);
            }
        } catch (e) {
            console.error('Unexpected error when claiming', e);
            if (e.message.includes('Can not sign transactions for account')) {
                // Someone has submitted the solution before the player!
                console.log("Oof, that's rough, someone already solved this.");
            }
        } finally {
            setShowLoader(false);
            // See if the transaction succeeded during transfer
            // or succeeded when creating a new account.
            // If unsuccessful, let the user try again.
            if (!transaction) {
                setClaimError(
                    "Couldn't transfer reward to that account, please try another account name or create a new one.",
                );
            } else {
                console.log('Transaction status:', transaction.status);
                const tx_succeeded = transaction.status.hasOwnProperty('SuccessValue');
                if (tx_succeeded) {
                    let tx_success_value = b64toUtf8(transaction.status.SuccessValue);
                    const actNextPuzzle = () => {
                        // This tells the React app that it's solved and claimed
                        setSolvedPuzzle(false);
                        setClaimError('');

                        // Clean up and get ready for next puzzle
                        localStorage.removeItem('playerSolvedPuzzle');
                        localStorage.removeItem('guesses');
                    };
                    if (needsNewAccount) {
                        if (tx_success_value) {
                            actNextPuzzle();
                        } else {
                            setClaimError('Could not create that account, please try another account name.');
                        }
                    } else {
                        if (tx_success_value) {
                            actNextPuzzle();
                        } else {
                            setClaimError(
                                "Couldn't transfer reward to that account, please try another account name or create a new one.",
                            );
                        }
                    }
                } else {
                    // Transaction failed
                    setClaimError(`Error with transaction: ${transaction.status.Failure}`);
                    console.log('Error with transaction', transaction.status.Failure);
                }

                if (transaction.hasOwnProperty('transaction') && transaction.transaction.hasOwnProperty('hash')) {
                    console.log('Transaction hash:', transaction.transaction.hash);
                }
            }
        }
    };

    const onCrosswordComplete = useCallback(async completeCount => {
        if (completeCount) {
            let gridData = createGridData(data).gridData;
            // console.log('gridData: ', gridData);
            loadGuesses(gridData, 'guesses');
            await checkSolution(gridData);
        }
    }, []);

    const checkSolution = async gridData => {
        console.log('data: ===> ', data);
        console.log('gridData: ===> ', gridData);
        const seedPhrase = parseSolutionSeedPhrase(data, gridData);
        console.log('seedPhrase: ===> ', seedPhrase);

        const { secretKey, publicKey } = parseSeedPhrase(seedPhrase);

        console.log('secretKey: ', secretKey);
        console.log('publicKey: ', publicKey);

        // Compare crossword solution's public key with the known public key for this puzzle
        // (It was given to us when we first fetched the puzzle in index.js)
        if (publicKey === crosswordSolutionPublicKey) {
            const keyStore = new nearAPI.keyStores.InMemoryKeyStore();
            const keyPair = nearAPI.utils.key_pair.KeyPair.fromString(secretKey);
            await keyStore.setKey(nearConfig.networkId, nearConfig.contractName, keyPair);
            nearConfig.keyStore = keyStore;
            const near = await nearAPI.connect(nearConfig);
            const crosswordAccount = await near.account(nearConfig.contractName);

            const playerPublicKey = playerKeyPair.publicKey;
            console.log('Unique public key for you as the player: ', playerPublicKey);

            let transaction;
            try {
                setShowLoader(true);
                transaction = await crosswordAccount.functionCall({
                    contractId: nearConfig.contractName,
                    methodName: 'submit_solution',
                    args: {
                        solver_pk: playerPublicKey,
                    },
                    gas: '300000000000000', // You may omit this for default gas
                    attachedDeposit: 0, // You may also omit this for no deposit}
                });
                isWindow && localStorage.setItem('playerSolvedPuzzle', crosswordSolutionPublicKey);
                setSolvedPuzzle(crosswordSolutionPublicKey);
            } catch (e) {
                if (e.message.contains('Can not sign transactions for account')) {
                    // Someone has submitted the solution before the player!
                    console.log("Oof, that's rough, someone already solved this.");
                }
            } finally {
                setShowLoader(false);
                console.log('Transaction status:', transaction.status);
                console.log('Transaction hash:', transaction.transaction.hash);
            }
        } else {
            console.log("That's not the correct solution. :/");
        }
    };

    const signIn = async () => {
        const walConn = await walletConnection();
        walConn.requestSignIn(
            nearConfig.contractName,
            '', // title. Optional, by the way
            '', // successUrl. Optional, by the way
            '', // failureUrl. Optional, by the way
        );
    };

    const signOut = async () => {
        const walConn = await walletConnection();
        walConn.signOut();
        window.location.replace(window.location.origin + window.location.pathname);
    };

    const header = (
        <>
            <Head>
                <title>VTS crossword puzzle</title>
                <meta name="description" content="Generated by create next app" />
                <link rel="icon" href="/favicon.png" />
            </Head>
            <header className="site-header">
                <div className="site-logo">
                    <Link href="/">
                        <a>
                            <Image
                                src="/imgs/logo_v2.png"
                                alt="Near Crossword Puzzle"
                                quality={100}
                                width={350}
                                height={103}
                            />
                        </a>
                    </Link>
                </div>
                <div id="login">
                    {currentUser ? (
                        <button onClick={signOut}>Log out</button>
                    ) : (
                        <button onClick={signIn}>Log in</button>
                    )}
                </div>
            </header>
        </>
    );

    let claimStatusClasses = 'hide';
    if (claimError) {
        claimStatusClasses = 'show';
    }

    if (showLoader) {
        return (
            <>
                {header}
                <main className="main-area">Loading...</main>
            </>
        );
    }

    if (data && !solvedPuzzle) {
        return (
            <>
                {header}
                <main className="main-area">
                    <Crossword data={data} onCrosswordCorrect={onCrosswordComplete} />
                </main>
            </>
        );
    }

    if (solvedPuzzle) {
        return (
            <>
                {header}
                <main className="main-area">
                    <WonPage
                        claimStatusClasses={claimStatusClasses}
                        claimError={claimError}
                        needsNewAccount={needsNewAccount}
                        setNeedsNewAccount={setNeedsNewAccount}
                        claimPrize={claimPrize}
                        playerKeyPair={playerKeyPair}
                        nearConfig={nearConfig}
                    />
                </main>
            </>
        );
    }

    if (!solvedPuzzle && !claimError) {
        return (
            <>
                {header}
                <main className="main-area">
                    <SuccessPage />
                </main>
            </>
        );
    }

    return (
        <>
            {header}
            <main className="main-area">
                <NoCrosswordsPage />
            </main>
        </>
    );
};

export default Home;
