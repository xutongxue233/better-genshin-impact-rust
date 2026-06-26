use super::*;

pub(crate) fn settle_promise_value(value: JsValue, context: &mut Context) -> Result<JsValue> {
    let Some(promise) = value
        .as_promise()
        .cloned()
        .and_then(|object| JsPromise::from_object(object).ok())
    else {
        return Ok(value);
    };
    settle_js_promise(promise, context)
}

pub(crate) fn settle_js_promise(promise: JsPromise, context: &mut Context) -> Result<JsValue> {
    context.run_jobs();
    match promise.state() {
        PromiseState::Fulfilled(value) => Ok(value),
        PromiseState::Rejected(reason) => Err(ScriptEngineError::JavaScript(js_value_to_string(
            &reason, context,
        ))),
        PromiseState::Pending => Err(ScriptEngineError::PromiseEvaluationPending),
    }
}

pub(crate) fn settle_promise_value_for_callback(
    value: JsValue,
    context: &mut Context,
) -> JsResult<JsValue> {
    settle_promise_value(value, context)
        .map_err(|err| JsNativeError::typ().with_message(err.to_string()).into())
}
