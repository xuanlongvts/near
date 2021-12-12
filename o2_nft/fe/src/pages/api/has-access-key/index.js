const hasAccessKey = (_, res) => {
    console.log('hasAccessKey:');
    res.status(200).json('hasAccessKey');
};

export default hasAccessKey;
