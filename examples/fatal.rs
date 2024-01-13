use loga::{
    fatal,
    new_err_with,
    ea,
    new_agg_err,
    new_err,
};

fn main() {
    fatal(
        new_agg_err(
            "The main thing failed",
            vec![
                new_err_with("The primary system exploded", ea!(att1 = "Hi", att2 = 423))
                    .also(
                        new_err_with(
                            "An incidental_error with a threateningly long message that might be able to wrap if I extend the length somewhat further and then some I guess going by editor width this might not be quite enough",
                            ea!(
                                another_attr =
                                    "This is a very long message, hopefully it gets wrapped somewhere between the start and the end of the screen"
                            ),
                        ),
                    )
                    .also(new_err("Nothing much else to add")),
                new_err("Just tacking this one on too")
            ],
        ),
    );
}
