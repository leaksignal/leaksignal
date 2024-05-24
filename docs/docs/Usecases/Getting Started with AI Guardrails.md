---
sidebar_position: 1
---

## Guardrails for LLMs

Securing data in modern architectures demands real-time visibility and control. LeakSignal empowers security teams by providing live traffic analysis to observe data access, mitigate threats, and maintain regulatory compliance.

### Key Features of LeakSignal

1. **Sensitive Data Visibility**
   - Achieve GenAI governance through detailed data visibility at the service level. LeakSignal detects abuse and prevents unknown data leaks.

2. **Threat Mitigation**
   - Real-time data classification identifies and blocks abusive behavior based on authenticated digital identities.

3. **Incident Response and Attestation**
   - Gain holistic, multi-protocol visibility into sensitive data flows with complete audit trails of accessed data.

### Setting Up Guardrails for LLMs with LeakSignal

**Step 1: Configure LeakSignal to Monitor LLM Outputs**
- Deploy LeakSignal within your architecture. [See deployment options](../Deployment/Istio%20&%20OSSM).
- Select or create the [appropriate LeakSignal policy](../Policy/Overview).

**Step 2: Tune Your Deployment**
- Test your policy with live interactions to ensure accuracy.
- Configure secondary classifiers to minimize false positives.

**Step 3: Enable Real-Time Monitoring and Alerts**
- Set up LeakSignal to notify relevant teams for immediate action.

**Step 4: (Optional) Implement Mitigation Actions**
- Automate responses to detected violations based on predefined policies.

### Policy Configuration Overview

LeakSignal's default LLM policy covers a wide range of patterns to ensure comprehensive protection:

**Sensitive Information Patterns:**
- Phone Numbers
- Email Addresses
- Credit Card Numbers
- Social Security Numbers

**Offensive or Harmful Language:**
- Lists of offensive terms and slurs.

**Misinformation and Fake News:**
- Keywords related to conspiracy theories and debunked claims.

**Illegal Activities:**
- Terms related to hacking, drugs, fraud, and other illegal activities.

**Self-Harm or Violence:**
- Terms associated with self-harm and violence.

**Malicious Code or Hacking Instructions:**
- Keywords related to malware, viruses, and hacking techniques.

**Explicit or Adult Content:**
- Terms related to adult content and sexual acts.

**Political or Propaganda Content:**
- Keywords associated with extreme political views and propaganda.

**False Accusations or Defamation:**
- Phrases indicating false accusations and defamatory language.

**Financial Scams and Fraud:**
- Phrases related to scams and fraud schemes.

### Additional Strategies

**Contextual Analysis**
- Use LeakSignal's NLP capabilities to understand context and reduce false positives. 

**Machine Learning Models**
- Leverage trained classifiers for more accurate detection of harmful content. 

**Blacklist and Whitelist Approaches**
- Regularly update blacklists and whitelists to refine detection.

**Human Review**
- Have human moderators review flagged outputs, especially ambiguous or context-sensitive content.

By implementing these steps and strategies, organizations can set up effective guardrails for LLMs, ensuring robust data security and compliance. This guide provides a starting point, with future documentation exploring advanced classifier options and NLP techniques for enhanced data protection.
