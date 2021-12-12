const handler = (req, res) => {
    console.log('handler hello world: ', req);
    res.status(200).json('hello world');
};

export default handler;
