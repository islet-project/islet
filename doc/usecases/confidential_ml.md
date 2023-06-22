# Confidential machine learning with ISLET

## Introduction

In order for traditional machine learning (ML) to work, *data provider* (e.g., mobile device) would have no choice but to give their data to *server* which offers the ability to run ML operations (inference and training). It apparently raises an issue of user data privacy because *data provider* might want to keep their data local.

Federated learning (FL), also known as privacy-preserving machine learning, has come to the rescue of this privacy issue by the concept of on-device training. More technically, in this way, it would no longer send user data to *server* but instead send *model parameters* derived as a result of on-device training.

On one hand, federated learning is a good thing for security, but on the other hand, practical issues are remaining, which hinder the wide adoption of it. The thing is, a lot of ML services have already been built over traditional ML, so it's never going to be easy to turn them into a federated learning compatible form. Some of them might want to stick to traditional ML as security could not be a big priority.

On top of it, there are still security issues remaining even when you use federated learning. For example, say that you are working for an AI company specializing in model development and your AI models have to be treated as private assets. But for machine learning to work, you have to send your model down to devices and trust they do not see or reveal your model. Also, there have been academic papers that demonstrate it's still possible to learn something about user data through adversarial ML attacks even in the federated learning setting (e.g., [paper-1](https://arxiv.org/abs/2003.14053), [paper-2](https://www.usenix.org/system/files/sec20summer_fang_prepub.pdf)).

These problems mentioned by far have led us to try to find out a practical yet general solution that can cover every type of machine learning (from traditional to federated).
And that solution is what we call *confidential machine learning* (shortly, *confidential ML*), which is based on *trusted execution environments (TEE)*. Admittedly, Confidential ML is not new and many companies working on TEE already have services that offer confidential ML but to some limited extent. Specifically, what most companies are caring about is only server-side aspects and device-side aspects have not been seriously considered, and thus they all fail to build an end-to-end confidential ML service.

In this article, we're going to explore how ISLET makes a unique addition to confidential ML to accomplish a real end-to-end confidential ML service,
ranging from traditional ML to federated learning.

## Issues in traditional ML

In traditional ML, there are two kinds of privacy leaks:
- *data leak*: devices have to send data in order for servers to do ML operations including inference and training. In this case, those servers can see user data without permission.
- *model leak*: to reduce latency and retrieve an ML outcome quickly, devices can download an ML model from servers and do on-device inferences using ML frameworks such as TensorFlow lite. In this case, devices can see an ML model that should be treated as secret to servers.

While most confidential computing platforms are targeting cloud servers (e.g., Intel SGX and AMD SEV), most ML clients come from mobile devices where confidential computing is out of reach at the time of this writing. Of course, most mobile devices based on ARM have TrustZone, which is one instance of TEE, but it is typically not allowed for 3rd party applications to get protected by TrustZone as it is built for vendor-specific confidential information.

The problem is, while *data leak* can easily be eliminated by running servers on confidential computing platforms such as Intel SGX or AMD SEV, we have no way to protect *model leak* against malicious or honest-but-curious devices. And this cannot be solved without the aid of a device-side confidential computing platform.

## ISLET for traditional ML

ISLET can tackle the above privacy issues that traditional ML has, by extending confidential computing to mobile devices.
There are three different components involved in this confidential traditional ML: (1) *model-provider*, (2) *device (data-provider)*, (3) *runtime*.

*model-provider* is the owner of AI models and wants to prevent any other components from seeing their models that contain confidential information.
*device* represents mobile devices and thus sends local data to *runtime* to benefit from ML services. We assume *device* here belongs to a specific vendor like Samsung.
Lastly, *runtime* provides ML functionalities and acts as a server. It takes data from *device* and model from *model-provider* and does actual ML stuff.
In this setting, *runtime* and *model-provider* can be running on confidential computing platforms that public cloud vendors such as Azure or Google cloud provide,
while *device* can be running on ISLET which is based on ARM CCA.

Roughly thinking, putting everything into the scope of confidential computing suffices to solve both *data leak* and *model leak*. But this holds true only if it's assumed that those three components mutually trust each other.
For example, *runtime* expects *device* to keep their AI models in ISLET and not to reveal them anywhere else. But, *device* can freely break that assumption and take AI models out of ISLET with the intent of uncovering how their models work, which raises *model leak*.

To prevent this from happening, we need one more assumption that all codes of those three components are open-sourced and therefore everyone can see and audit them.
If *runtime* is programmed to keep user data local and it is open-sourced, we can assure that in no circumstances will *data leak* happen. It is the same in *model leak*.
To check if a program that is about to launch matches what we expect (e.g., *runtime* that doesn't leak user data), we need to take the measurement of that program and associate that with the attestation process. We use [Certifier framework](https://github.com/vmware-research/certifier-framework-for-confidential-computing) to realize this verification as that framework is designed to work across various TEEs in a unified manner, satisfying our requirement.

## Issues in federated learning

As mentioned earlier, in federated learning, someone may think *data leak* will not happen as training is done on the device side, and thus user data never leaves devices. But this assumption has turned out to be wrong as several attacks have demonstrated that malicious servers (or honest-but-curious servers) can infer user data from model parameters (i.e., weights). For example, [this paper](https://arxiv.org/abs/2003.14053) proposed an attack that extrapolates what data was used in on-device training from model parameters and successfully demonstrated it could recover some images used. (which is called *inversion attack*)

There is another kind of attack, which is called *inference attack*, that malicious devices can launch. A malicious device might want to know what data is used in another device. In federated learning, each device can download new global models, which reflect training results from all devices. This means that a newly downloaded model has information about data from other devices, which is what a malicious device can exploit for inference attacks.
By doing a comparison between two global models in a row, a malicious device can learn something about another device's data, for example, whether an image used in training includes someone on glasses or not.

One more interesting attack is what is called *poisoning attack*. As the name suggests, some devices can train with a large number of fake data in order to poison a global model in attackers' favor. For example, attackers might want to induce a victim to visit a specific place in order to leak the victim's private information in person. To do so, they can generate tons of data that would lead AI models to recommend visiting a specific place no matter what users ask.

## ISLET for federated learning

To stop the aforementioned attacks (*inversion attack*, *inference attack*, and *poisoning attack*), we can take the same approach as we did with traditional ML. The only difference would have to do with *runtime* as runtime servers would not do ML operations (training and inference) in federated learning. Instead, they do a so-called aggregation algorithm to build a new global model from local models that each device sends up.

For the former two attacks (inversion and inference), the open-source based security argument could still work in the same way. This is because, to launch them, attackers have to run programs that contain codes to do those attacks, leading to a measurement that is different than the allowed ones.

As for the last one (poisoning), it could not get protected in the same way as what actually breaks security comes from "data" not "program". In other words, even if *device* is authentic, a model could be trained with fake data, considering an attacker who is capable of taking control of "data" fed into the model.
We see that this attack could be addressed to some extent by designing and implementing peripheral protections (e.g., building a secure channel from keyboard hardware all the way to a secure application) on top of CCA primitives.

## Time to play around with real examples

Anyone can try out what we've explained so far, that is to say, running traditional ML or federated learning on top of ISLET with simple ML models.
Check out [this markdown file](https://github.com/Samsung/islet/tree/main/examples/confidential-ml) to play around with ISLET for confidential ML!
