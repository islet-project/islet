# Confidential Machine Learning

## What is confidential machine learning and why it matters?

In order for traditional machine learning (ML) to work, *data provider* (e.g., mobile device) would have no choice but to give their data to *server* which offers the ability to run ML operations (inference and training). It apparently raises an issue of user data privacy because *data provider* might want to keep their data local.

Federated learning has come to the rescue of this privacy issue by the concept of on-device training. More technically, with this way, it would no longer send user data to *server* instead send *model parameters* derived as a result of on-device training.

However, there are still security issues remaining even when you use federated learning. For example, say that you are working for a AI company specializing in model development and your AI models have to be treated as private asset. But for machine learning to work, you have to send your model down to devices and trust they do not see or reveal your model.

On top of that, there have been academic papers that demonstrate it's still possible to learn something about user data through adversarial ML attacks even in the federated learning setting (e.g., [paper-1](https://arxiv.org/abs/2003.14053), [paper-2](https://www.usenix.org/system/files/sec20summer_fang_prepub.pdf)).

This has led us to try to find out a practical solution, after an investigation, we have drawn a conclusion that "confidential computing" would be a good fit for "federated learning (FL, shortly)" (a traditional machine learning as well) and actually help solving the above problems.

## Approach

The approach to make ML/FL completely secure is to open-source all codes of components involved in ML/FL and verify those components via attestation. Then, each party can see other party's code thus can trust that others will not reveal others' data and that their data never goes outside TEE.

There are four components involved:
- *certifier-service*: takes a role of attestation and checking a pre-written policy.
- *runtime*: provides ML functionalities and acts as a server.
- *model-provider*: is the owner of AI models and sends models to *runtime*.
- *device (data provider)*: sends local data to *runtime* in ML setting. It sends model parameters to *runtime* in FL setting.

In ML setting, it works as following:
```
1. <model-provider> sends a model to runtime.
2. <runtime> receives a model.
3. <device> sends local data to runtime and asks runtime to do training with that.
4. <runtime> receives device's data and trains with it and then sends a newly trained model to the device.
5. <device> do inference with a newly trained model.
```

In FL setting, it works as following:
```
1. <model-provider> sends a model to runtime.
2. <runtime> receives a model.
3. <device> do training with local data and give runtime a trained model (local model).
4. <runtime> receives local model from multiple devices and do aggregation to build a global model and sends the global model to devices.
5. <device> do inference with the global model.
```

In both cases, every components do not contain codes that try to leak other party's private asset to anywhere but inside TEE.
And this argument can be guaranteed by putting the measurement of their codes into a policy that is maintained by *[certifier framework](https://github.com/vmware-research/certifier-framework-for-confidential-computing)* and using the policy throughout the attestation process.

To get a feeling why it is secure, imagine an attacker attempts to pretend it is *runtime* and leak user data to somewhere else. It will not be allowed to pass attestation because such malicious *runtime* codes are not specified in the policy. It is the same in the device side.

An attacker might want to make an arbitrary local model that is different than a genuine local model derived from training process, which is referred to as "local model poisoning attack", in order to change a global model in an attacker's favor. To do so, the attacker has to build an application containing that attack codes but that application will be prohibited to launch due to the policy check.

[TODO] Note that as of now we use the same measurement for those four components just for testing purpose. We need to build a script to compute a proper measurement for each component at compile-time.

## Model

We build two simple RNN-based models,

(1) *code generation*:
This code generation model is currently not based on transformer models on which the state-of-the-art models such as ChatGPT rely.
Instead, it simply works like a text classification model that takes a sentence like "write a function to add two numbers" and matches it to one of functions in a pre-defined list.
See [this](./CODE_MODEL.md) to know instructions to play around with this model.
If you are interested in what this model is like, see [this python script](./model-provider/model_code.py).

(2) *word prediction*: (Note that this model is not actively supported as of now.)
It takes three characters as input and makes a prediction of five word letter, that is, it aims to predict the following two characters.
For example, if you type in "abo" as input, this model may output "about".
See [this](./WORD_MODEL.md) to know instructions to play around with this model.
If you are interested in what this model is like, see [this python script](./model-provider/model.py).

As these two models are not build to show how good they are in terms of ML accuracy,
we think they are good enough to prove the concept of Islet.